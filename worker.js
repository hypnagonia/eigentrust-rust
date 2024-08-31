let e

(async () => {
    e = await import('./pkg/eigentrust.js')
    await e.default()
})()

self.onmessage = async function (event) {
    const { localtrustBytes, pretrustBytes } = event.data
    console.time("eigentrust job")
    const result = e.run(localtrustBytes, pretrustBytes, 0.5)
    console.timeEnd("eigentrust job")
    self.postMessage(result)
}
