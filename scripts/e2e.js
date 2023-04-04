"use strict"

// This is effectively just a giant smoke test of the whole system.

const path = require("path")
const fs = require("fs")
const crypto = require("crypto")
const child_process = require("child_process")
const http = require("http")
const os = require("os")

/** @returns {never} */
function bail(msg) {
    console.error(msg)
    process.exit(1)
}

function integerArg(arg, invalid) {
    const result = Number.parseInt(arg)
    if (Number.isNaN(result)) bail(invalid)
    return result
}

const args = {
    // Use a request timeout of 5 seconds by default.
    reqTimeout: 5,

    // Default the key directory to a `/tmp` directory.
    keyDir: "/tmp/integ-test.keys",

    // Limit the test duration to 1 minute by default, but leave it configurable for local testing.
    testDuration: 60,

    // Limit the test interval to 5 seconds, but leave it configurable for testing.
    testInterval: 5,

    port: 8080,

    binary: path.resolve("target/release/journald-exporter"),
}

let argName

for (const arg of process.argv.slice(2)) {
    if (argName) {
        switch (argName) {
        case "-c":
            args.reqTimeout = integerArg(arg, "Request timeout must be a number if provided")
            break

        case "-k":
            if (!arg) bail("Key directory must not be empty.")
            args.keyDir = path.resolve(arg)
            break

        case "-p":
            args.port = integerArg(arg, "Port must be a number if provided")
            if (args.port < 1 || args.port > 65535) bail("Port must be within 1 and 65535 inclusive")
            break

        case "-d":
            args.testDuration = integerArg(arg, "Test duration must be a number if provided")
            if (args.testDuration < 1) bail("Test duration must be a positive number of seconds")
            break

        case "-i":
            args.testInterval = integerArg(arg, "Test interval must be a number if provided")
            if (args.testInterval < 1) bail("Test interval must be a positive number of seconds")
            break

        case "-b":
            if (!arg) bail("Release binary path must not be empty.")
            args.binary = path.resolve(arg)
            break

        default:
            if (argName) bail(`Expected a value for argument \`${argName}\``)
        }
        argName = undefined
    } else {
        if (!/^-[ckpdib]$/.test(arg)) bail(`Unknown argument \`${arg}\``)
        argName = arg
    }
}

if (argName) bail(`Expected a value for argument \`${argName}\``)
if (process.getuid() !== 0) bail("This script must run as root")

function reportAsyncError(e) {
    if (e && e.code !== "ABORT_ERR") {
        console.error(`[INTEG] Error thrown: ${e.stack}`)
        if (!process.exitCode) process.exitCode = 1
    }
}

function safeAbort(ctrl) {
    try {
        ctrl.abort()
    } catch (e) {
        reportAsyncError(e)
    }
}

const ctrl = new AbortController()
process.on("SIGTERM", () => { safeAbort(ctrl) })
process.on("SIGINT", () => { safeAbort(ctrl) })

function cleanup() {
    fs.rm(args.keyDir, {recursive: true, force: true}, reportAsyncError)
}

// Generate a simple static key for testing.
const testKey = crypto.randomBytes(16).toString("hex")

function fetchLoop(parentSignal, terminateHandler) {
    // Give the server time to boot up. (It's normally near instant, so it shouldn't take long.)
    let byteCount = 0
    let contentType, statusCode, timer, req

    function cleanupFetch() {
        clearTimeout(timer)
        parentSignal.removeEventListener("abort", onAbort)
        if (req) req.destroy()
        terminateHandler()
    }

    function onAbort() {
        console.log("[INTEG] Request aborted")
        cleanupFetch()
    }

    function onTimeout() {
        console.log("[INTEG] Request timed out")
        cleanupFetch()
    }

    function loopError(e) {
        console.log("[INTEG] Request errored")
        cleanupFetch()
        if (!(req && req.destroyed) && !/^EPIPE$|^ECONN(?:ABORTED|REFUSED|RESET)$/.test(e)) {
            reportAsyncError(e)
        }
    }

    function reqFinished() {
        if (!req) return
        req = undefined
        clearTimeout(timer)
        console.log(`[INTEG] Response: ${statusCode} ${http.STATUS_CODES[statusCode]} ${contentType} ${byteCount}B`)
        if (!statusCode || statusCode < 200 || statusCode > 299) {
            return loopError(new Error(`Received unsuccessful response with status ${statusCode}`))
        }
        if (!contentType || !contentType.includes("application/openmetrics-text")) {
            return loopError(new Error(`Received response with content type ${contentType}`))
        }
        if (!byteCount) {
            return loopError(new Error("Received empty response"))
        }
        byteCount = 0
        if (!parentSignal.aborted) timer = setTimeout(loop, args.testInterval * 1000)
    }

    function loop() {
        timer = setTimeout(onTimeout, args.reqTimeout * 1000)

        req = http.get(`http://localhost:${args.port}/metrics`, {
            headers: {
                authorization: `Basic ${Buffer.from(`metrics:${testKey}`, "binary").toString("base64")}`
            },
        })

        req.once("error", loopError)
        req.once("response", res => {
            byteCount = 0
            statusCode = res.statusCode
            contentType = res.headers["content-type"]
            res.on("data", buf => { byteCount += buf.length })
            res.on("error", loopError)
            res.on("end", reqFinished)
        })

        req.end()
    }

    parentSignal.addEventListener("abort", onAbort, {once: true})
    loop()
}

function runChildTest() {
    const child = child_process.spawn(
        "systemd-run",
        [
            "--wait", "--quiet", "--pty", "--pipe", "--collect",
            "--property=Type=notify",
            "--property=WatchdogSec=5s",
            "--property=TimeoutStartSec=5s",
            args.binary, "--port", args.port, "--key-dir", args.keyDir,
        ],
        {stdio: "inherit"},
    )

    let fetchCtrl = new AbortController()
    let terminationAttempted = false
    let fetchTimer

    function terminateHandler() {
        if (terminationAttempted) return
        terminationAttempted = true
        clearTimeout(fetchTimer)
        safeAbort(fetchCtrl)

        const killTimer = setTimeout(() => child.kill("SIGKILL"), 2000)
        child.once("exit", (code, signal) => {
            clearTimeout(killTimer)
            console.log("[INTEG] Child exited")
            if (code) {
                process.exitCode = code
            } else if (signal) {
                process.exitCode = 128 + os.constants.signals[signal]
            }
            cleanup()
        })
        child.once("error", e => {
            clearTimeout(killTimer)
            console.log("[INTEG] Child errored")
            reportAsyncError(e)
            cleanup()
        })
        child.kill("SIGTERM")
        console.log("[INTEG] Child terminate signal sent")
    }

    ctrl.signal.addEventListener("abort", terminateHandler, {once: true})

    function startFetch() {
        console.log("[INTEG] Starting fetch loop")

        fetchTimer = setTimeout(() => {
            ctrl.signal.removeEventListener("abort", terminateHandler)
            terminateHandler()
        }, args.testDuration * 1000)

        fetchLoop(fetchCtrl.signal, terminateHandler)
    }

    function onSpawn() {
        child.off("error", onError)
        console.log(`[INTEG] Child PID: ${child.pid}`)
        fetchTimer = setTimeout(startFetch, 2000)
    }

    function onError(e) {
        child.off("spawn", onSpawn)
        reportAsyncError(e)
    }

    child.once("error", onError)
    child.once("spawn", onSpawn)
}

fs.rm(args.keyDir, {recursive: true, force: true}, err => {
    if (err) return reportAsyncError(err)
    fs.mkdir(args.keyDir, {recursive: true, mode: 0o755}, err => {
        if (err) return reportAsyncError(err)
        ctrl.signal.addEventListener("abort", cleanup, {once: true})

        fs.writeFile(path.join(args.keyDir, "test.key"), testKey, {
            flag: "wx",
            mode: 0o600,
            signal: ctrl.signal,
        }, err => {
            if (err) reportAsyncError(err)
            else runChildTest()
        })
    })
})
