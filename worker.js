let e

(async () => {
    e = await import('./eigentrust_rs.js')
    await e.default()
    await e.prepare()
})()

self.onmessage = async function (event) {
    const { localtrustBytes, pretrustBytes } = event.data
    console.time("eigentrust job")
    const result = e.run(localtrustBytes, pretrustBytes, 0.5)
    console.timeEnd("eigentrust job")
    self.postMessage(result)
}
