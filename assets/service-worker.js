import init, { run } from './pkg/eigentrust.js'
    
/*(async () => {
    eigentrustModule = await import('./pkg/eigentrust.js');
    await eigentrustModule.default();
})();
*/

// const e = importScripts('./pkg/eigentrust.js');

self.addEventListener('install', event => {
    console.log('Service Worker installing.');
});

self.addEventListener('activate', event => {
    console.log('Service Worker activating.');
});


self.addEventListener('message', async function (event) {
    console.log('Service Worker started');
    // console.log(e)
    await init()
    const { localtrustBytes, pretrustBytes, alpha } = event.data;
    
    console.time("eigentrust job");
    const result = run(localtrustBytes, pretrustBytes, alpha);
    console.timeEnd("eigentrust job");
    console.log('Service Worker finished');
    event.ports[0].postMessage(result);
});
