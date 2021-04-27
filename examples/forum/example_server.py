import socket
import sys
import socketserver
from http.server import BaseHTTPRequestHandler
from http.server import HTTPServer
import urllib.parse
import forum_util as ut

# using port numbers prepended with 9 to avoid calling sudo during test
servport = 9090
posts_file = "./posts/posts.txt"


def getCommentsFromUser(fd, usr):
    output_txt = "["
    comment = ""
    try:
        comment = fd.read()
    except:
        err = "Error reading file {}".format(posts_file)
        print(err)
        raise Exception(err)
    cnt = 0
    offset = 0
    while True:
        cmt_json = ut.read_simple_json(comment, offset)
        if(cmt_json == None):
            break
        offset += (cmt_json['_offset'] + cmt_json['_length']) # relative positional seek
        if(usr == None or cmt_json['user'] == usr):
            cnt += 1
            output_txt += (ut.jsonify_urllib_params(cmt_json) + ",")
    print("Number of matches:", cnt)
    output_txt = output_txt[0:-1] + "]" # replace newline at the end of last comment
    if(cnt == 0):
        return None
    return output_txt

# https://wiki.python.org/moin/BaseHttpServer
class MyHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        print("")
        print("GET for", self.client_address)
        print("Path:", self.path)

        status = 200
        status_msg = "Ok"
        output_txt = ""
        data_dict = {}
        try:
            # params is everything after "?"
            params = self.path[self.path.index("?")+1:]
            data_dict = urllib.parse.parse_qs(params)
        finally:
            print("Request params:", data_dict)

        try:
            with open(posts_file, "r") as fd:
                # If no user specified, get all comments.
                if not 'user' in data_dict:
                    output_txt = getCommentsFromUser(fd, None)
                # Fetch comments from specified user
                else:
                    output_txt = getCommentsFromUser(fd, data_dict['user'][0])
                    if(output_txt == None):
                        output_txt = "This user has no comments."
        except:
            status = 500
            status_msg = "Error Retrieving Data -- File read fail or may not exist."
            print(status_msg + "\n")

        self.send_response(status, status_msg)
        if status == 200:
            self.send_header("Content-Type", "application/json")
        self.end_headers()
        if status == 200:
            self.wfile.write(output_txt.encode('utf-8'))
        

    # currently just act kinda like an echo server
    def do_POST(self):
        status = 200
        status_msg = "Ok"
        json_to_send = ""
        print("")
        print("POST for", self.client_address)
        
        try:
            data = self.rfile.read(int(self.headers.get('Content-Length'))).decode('utf-8')
            print(data)
            # this handles weird url special character stuff, turns params into dict
            data_dict = urllib.parse.parse_qs(data)
            print(data_dict)
            json_to_send = ut.jsonify_urllib_params(data_dict)
        except:
            status = 500
            status_msg = "Error reading POST data"
            print(status_msg + "\n")

        #append comment to forum post file
        try:
            with open(posts_file, "a") as fd:
                fd.write(json_to_send + "\n")
        except:
            status = 500
            status_msg = "Error posting comment"
            print(status_msg + "\n")

        self.send_response(status, "Server Error: " + status_msg)
        if status == 200:
            self.send_header("Content-Type", "text/plain")
        self.end_headers()
        if status == 200:
            self.wfile.write(("Message Posted:\n" + json_to_send).encode('utf-8'))


# https://stackoverflow.com/questions/19434947/python-respond-to-http-request
# https://docs.python.org/3/library/http.server.html
def httpServer(port):
    print("Starting HTTP Server on port", port)
    server_class = HTTPServer
    httpd = server_class(("", port), MyHandler)
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print('stopping server')
    finally:
        httpd.shutdown()
        httpd.server_close()
    return


if __name__ == '__main__':
    port = servport
    httpServer(port)
