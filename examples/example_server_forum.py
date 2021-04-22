import socket
import sys
import socketserver
from http.server import BaseHTTPRequestHandler
from http.server import HTTPServer
import urllib.parse

# using port numbers prepended with 9 to avoid calling sudo during test
servport = 9090

def jsonify(user, msg):
    max_chars = 256
    num_attributes = 2
    max_chars -= (num_attributes*6+1) # "":"", and {} but last line has no comma
    return "{\"user\":\"" + user + "\",\"msg\":\"" + msg + "\"}";

# https://wiki.python.org/moin/BaseHttpServer
class MyHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        print("")
        print("GET for", self.client_address)
        self.send_response(200, message="Ok")
        self.send_header("Content-Type", "text/plain")
        self.end_headers()
        self.wfile.write(jsonify("John", "Response").encode('utf-8'))

    # currently just act kinda like an echo server
    def do_POST(self):
        print("")
        print("POST for", self.client_address)
        print("Request line:", self.requestline)
        #print(self.headers) # stuff like Host and Content-Type
        data = self.rfile.read(int(self.headers.get('Content-Length'))).decode('utf-8')
        print(data)
        #print("Printed POST data")

        # this handles weird url special character stuff, turns params into dict
        data_dict = urllib.parse.parse_qs(data)
        print(data_dict)

        '''
        start_index = data.index("=")+1
        rest = data[start_index:]
        user = data[start_index:start_index+rest.index("&")]
        data = rest
        start_index = data.index("=")+1
        rest = data[start_index:]
        msg = data[start_index:]
        '''
        user = data_dict['user'][0]
        msg = data_dict['msg'][0]

        #print("User is", user, "and msg is", msg)

        self.send_response(200, message="Ok")
        self.send_header("Content-Type", "text/plain")
        self.end_headers()
        self.wfile.write(jsonify(user, msg).encode('utf-8'))


# https://stackoverflow.com/questions/19434947/python-respond-to-http-request
# https://docs.python.org/3/library/http.server.html
def httpServer(port):
    print("Starting HTTP Server")
    #server_class = socketserver.TCPServer
    server_class = HTTPServer
    #httpd = socketserver.TCPServer(("", port), MyHandler)
    httpd = server_class(("", port), MyHandler)
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print('stopping client')
    finally:
        httpd.shutdown()
        httpd.server_close()
    return


if __name__ == '__main__':
    port = servport
    httpServer(port)
