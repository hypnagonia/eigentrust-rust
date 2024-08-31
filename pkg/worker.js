let e

(async () => {
    e = await import('./eigentrust_rs.js')
    await e.default()
    await e.prepare()
})()

self.onmessage = async function (event) {
    const { localtrustBytes, pretrustBytes, alpha } = event.data
    console.time("eigentrust job")
    const result = e.run(localtrustBytes, pretrustBytes, alpha)
    console.timeEnd("eigentrust job")
    self.postMessage(result)
}
