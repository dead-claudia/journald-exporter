"use strict"

// This is effectively just a giant smoke test of the whole system.

const path = require("path")
const fs = require("fs")
const crypto = require("crypto")
const child_process = require("child_process")
const http = require("http")
const os = require("os")
const readline = require("readline")

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

// Limit the test interval to 5 seconds
const TEST_INTERVAL = 5000

// Use a request timeout of 5 seconds.
const REQUEST_TIMEOUT = 5000

const args = {
    // Default the key directory to a `/tmp` directory.
    keyDir: "/tmp/integ-test.keys",

    // Limit the test duration to 1 minute by default, but leave it configurable for local testing.
    testDuration: 60,

    port: 8080,

    binary: path.resolve("target/release/journald-exporter"),
}

let argName

for (const arg of process.argv.slice(2)) {
    if (argName) {
        switch (argName) {
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

        case "-b":
            if (!arg) bail("Release binary path must not be empty.")
            args.binary = path.resolve(arg)
            break

        default:
            if (argName) bail(`Expected a value for argument \`${argName}\``)
        }
        argName = undefined
    } else {
        if (!/^-[kpdb]$/.test(arg)) bail(`Unknown argument \`${arg}\``)
        argName = arg
    }
}

if (argName) bail(`Expected a value for argument \`${argName}\``)
if (process.getuid() !== 0) bail("This script must run as root")

function reportAsyncError(e) {
    if (
        e &&
        e.type !== "abort" &&
        !/^ABORT_ERR$|^EPIPE$|^ECONN(?:ABORTED|REFUSED|RESET)$/.test(e.code)
    ) {
        console.error("[INTEG] Error thrown:", e)
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

// Generate a simple static key for testing.
const testKey = crypto.randomBytes(16).toString("hex")

function fetchLoop(parentSignal, terminateHandler) {
    // Give the server time to boot up. (It's normally near instant, so it shouldn't take long.)
    let byteCount = 0
    let timer, req, res

    function cleanupFetch(msg, e) {
        console.error(msg)
        reportAsyncError(e)
        clearTimeout(timer)
        parentSignal.removeEventListener("abort", onAbort)
        if (res) {
            res.off("data", onData)
            res.off("error", loopError)
            res.off("end", reqFinished)
        }
        if (req) {
            req.off("error", loopError)
            req.off("response", onResponse)
            req.destroy()
        }
        terminateHandler()
    }

    const onAbort = cleanupFetch.bind(null, "[INTEG] Request aborted")
    const onTimeout = cleanupFetch.bind(null, "[INTEG] Request timed out")
    const loopError = cleanupFetch.bind(null, "[INTEG] Request errored")

    function reqFinished() {
        if (!req) return
        req.off("error", loopError)
        res.off("data", onData)
        res.off("error", loopError)
        res.off("end", reqFinished)
        const statusCode = res.statusCode
        const contentType = res.headers["content-type"]
        req = res = undefined
        clearTimeout(timer)
        console.error(`[INTEG] Response: ${statusCode} ${http.STATUS_CODES[statusCode]} ${contentType} ${byteCount}B`)
        if (!statusCode || statusCode < 200 || statusCode > 299) {
            loopError(new Error(`Received unsuccessful response with status ${statusCode}`))
        } else if (!contentType || !contentType.includes("application/openmetrics-text")) {
            loopError(new Error(`Received response with content type ${contentType}`))
        } else if (!byteCount) {
            loopError(new Error("Received empty response"))
        } else {
            byteCount = 0
            if (!parentSignal.aborted) timer = setTimeout(loop, TEST_INTERVAL)
        }
    }

    function onData(buf) {
        byteCount += buf.length
    }

    function onResponse(response) {
        byteCount = 0
        res = response
        res.on("data", onData)
        res.once("error", loopError)
        res.once("end", reqFinished)
    }

    function loop() {
        timer = setTimeout(onTimeout, REQUEST_TIMEOUT)

        req = http.get(`http://localhost:${args.port}/metrics`, {
            headers: {
                authorization: `Basic ${Buffer.from(`metrics:${testKey}`, "binary").toString("base64")}`
            },
        })

        req.once("error", loopError)
        req.once("response", onResponse)
        req.end()
    }

    parentSignal.addEventListener("abort", onAbort, {once: true})
    loop()
}

function cleanup() {
    console.error("[INTEG] Cleaning up")
    fs.rm(args.keyDir, {recursive: true, force: true}, e => {
        reportAsyncError(e)
        console.error("[INTEG] Cleaned up")
        setTimeout(process.exit, 1)
    })
}

function runChildTest() {
    console.error("[INTEG] Spawning child")

    let journalctlCtrl = new AbortController()
    let fetchCtrl = new AbortController()
    let killCtrl = new AbortController()
    let terminationAttempted = false
    let stderr = []
    let fetchTimer, unitName

    const child = child_process.spawn(
        "systemd-run",
        [
            "--wait",
            "--collect",
            "--property=Type=notify",
            "--property=WatchdogSec=5s",
            "--property=TimeoutStartSec=5s",
            args.binary, "--port", args.port, "--key-dir", args.keyDir,
        ],
        {stdio: ["ignore", "inherit", "pipe"], signal: killCtrl.signal},
    )

    function terminateUnit() {
        // Don't care about if/when it exits.
        child_process.spawn("systemctl", ["stop", unitName], {stdio: "inherit"})
            .on("error", reportAsyncError)
    }

    function terminateHandler() {
        if (terminationAttempted) return
        terminationAttempted = true
        clearTimeout(fetchTimer)
        safeAbort(fetchCtrl)

        if (unitName) terminateUnit()
        else safeAbort(killCtrl)

        console.error("[INTEG] Child terminate signal sent")
    }

    function startFetch() {
        console.error("[INTEG] Starting fetch loop")

        fetchTimer = setTimeout(() => {
            ctrl.signal.removeEventListener("abort", terminateHandler)
            terminateHandler()
        }, args.testDuration * 1000)

        fetchLoop(fetchCtrl.signal, terminateHandler)
    }

    function detachOutput() {
        // Flush standard error buffer
        if (stderr) {
            for (const line of stderr) console.error(line)
            stderr = undefined
        }
    }

    function reportExitStatus(code, signal) {
        if (code) {
            process.exitCode = code
        } else if (signal) {
            process.exitCode = 128 + os.constants.signals[signal]
        }
    }

    function runErrorDisplayCommand(cmd, args) {
        try {
            const result = child_process.spawnSync(cmd, args, {encoding: "utf-8"})
            console.error(result.stdout.trimEnd())
            console.error(result.stderr.trimEnd())
            reportExitStatus(result.code, result.signal)
        } catch (e) {
            reportAsyncError(e)
        }
    }

    function checkUnitLive(line) {
        const exec = /^Running as unit:\s*([A-Za-z0-9@_-]+\.service)\b/.exec(line)
        if (!exec) return false
        unitName = exec[1]
        console.error(`[INTEG] Detected transient unit name: ${unitName}`)
        detachOutput()
        fetchTimer = setTimeout(startFetch, 2000)
        // Just spawn and forget. It's just for visibility, but it needs to run in parallel in the
        // background.
        child_process.spawn(
            "journalctl",
            ["--unit", unitName, "--follow", "--output=cat"],
            {stdio: "inherit", signal: journalctlCtrl.signal}
        ).on("error", reportAsyncError)
        return true
    }

    function checkUnitFailed(line) {
        const exec = /^Job for ([A-Za-z0-9@_-]+\.service) failed\b/.exec(line)
        if (!exec) return false
        const unit = exec[1]
        console.error(`[INTEG] Unit failed to initialize: ${unit}`)
        detachOutput()
        // Print for visibility. Doing it this way makes it much easier to sequence the two error
        // outputs.
        runErrorDisplayCommand("journalctl", ["--unit", unit, "--catalog", "--output=cat"])
        runErrorDisplayCommand("systemctl", ["status", unit])
        return true
    }

    // Just ignore this line
    function checkUnitFailedDetails(line) {
        return /^See "systemctl status[^"]*" and "journalctl[^"]*" for details\b/.test(line)
    }

    function onLine(line) {
        if (!stderr) {
            console.error(line)
        } else {
            if (checkUnitLive(line)) return
            if (checkUnitFailed(line)) return
            if (checkUnitFailedDetails(line)) return
            stderr.push(line)
        }
    }

    function onError(e) {
        console.error("[INTEG] Child errored")
        reportAsyncError(e)
        handleTermination()
    }

    function onExit(code, signal) {
        console.error(`[INTEG] Child exited with code ${code}, signal ${signal}`)
        detachOutput()
        reportExitStatus(code, signal)
        handleTermination()
    }

    function handleTermination() {
        safeAbort(journalctlCtrl)
        child.off("error", onError)
        child.off("exit", onExit)
        setTimeout(cleanup, 1000);
    }

    ctrl.signal.addEventListener("abort", terminateHandler, {once: true})
    // No longer needed - cleanup will happen upon termination or error.
    ctrl.signal.removeEventListener("abort", cleanup)

    const rl = readline.createInterface({
        input: child.stderr,
        crlfDelay: Infinity,
    })

    rl.on("error", reportAsyncError)
    rl.on("line", onLine)

    child.once("error", onError)
    child.once("exit", onExit)
}

ctrl.signal.addEventListener("abort", cleanup, {once: true})
fs.rm(args.keyDir, {recursive: true, force: true}, err => {
    if (err) {
        reportAsyncError(err)
        return
    }

    fs.mkdir(args.keyDir, {recursive: true, mode: 0o755}, err => {
        if (err) {
            reportAsyncError(err)
            return
        }

        fs.writeFile(path.join(args.keyDir, "test.key"), testKey, {
            flag: "wx",
            mode: 0o600,
            signal: ctrl.signal,
        }, err => {
            if (err) {
                reportAsyncError(err)
                return
            }

            runChildTest()
        })
    })
})
