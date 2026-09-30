// Runs EigenTrust off the main thread.
// Uses the multithreaded build when the page is cross-origin isolated
// (SharedArrayBuffer available), the single-threaded one otherwise.

const parallel = self.crossOriginIsolated && typeof SharedArrayBuffer !== 'undefined'

const ready = (async () => {
    if (parallel) {
        try {
            const m = await import('./pkg-parallel/eigentrust.js')
            await m.default()
            const threads = Math.max(1, Math.min(navigator.hardwareConcurrency || 4, 16))
            // if the thread workers cannot start, initThreadPool never settles
            await Promise.race([
                m.initThreadPool(threads),
                new Promise((_, reject) => setTimeout(() => reject(new Error('thread pool timeout')), 8000)),
            ])
            return { run: m.run, threads }
        } catch (e) {
            console.warn('parallel build unavailable, falling back', e)
        }
    }
    const m = await import('./pkg/eigentrust.js')
    await m.default()
    return { run: m.run, threads: 1 }
})()

const encoder = new TextEncoder()

self.onmessage = async ({ data }) => {
    const { id, localtrust, pretrust, alpha } = data
    try {
        const { run, threads } = await ready
        const lt = typeof localtrust === 'string' ? encoder.encode(localtrust) : localtrust
        const pt = typeof pretrust === 'string' ? encoder.encode(pretrust) : pretrust
        const t0 = performance.now()
        const out = JSON.parse(run(lt, pt, alpha))
        const ms = performance.now() - t0
        if (out.Err !== undefined) self.postMessage({ id, error: out.Err, threads })
        else self.postMessage({ id, scores: out.Ok, ms, threads })
    } catch (e) {
        self.postMessage({ id, error: String(e && e.message || e), fatal: true })
    }
}
