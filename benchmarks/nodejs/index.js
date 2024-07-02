import * as http from 'node:http'

let server = http.createServer((req, res) => {
  res.write("Hello World!")
  res.end()
})

server.listen(8080)