let e

(async () => {
    e = await import('./pkg/eigentrust.js')
    await e.default()
})()

self.onmessage = async function (event) {
    const { localtrustBytes, pretrustBytes, alpha } = event.data
    console.time("eigentrust job")
    const result = e.run(localtrustBytes, pretrustBytes, alpha)
    console.timeEnd("eigentrust job")
    self.postMessage(result)
}
