import { App } from 'uWebSockets.js';
const port = 8080;

App()
    .get('/', (res, req) => {
        res.cork(() => {
            res.writeHeader('content-type', 'text/html').end(
                'hello world'
            )
        })
    })
    .listen(port, (token) => {
        if (token) {
            console.log('Listening to port ', port);
        }
    })