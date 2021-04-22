import socket
import sys
import socketserver
from http.server import BaseHTTPRequestHandler
from http.server import HTTPServer
import urllib.parse

# using port numbers prepended with 9 to avoid calling sudo during test
servport = 9090
posts_file = "./forum/posts.txt"

def jsonify(user, msg):
    max_chars = 256
    num_attributes = 2
    max_chars -= (num_attributes*6+1) # "":"", and {} but last line has no comma
    return "{\"user\":\"" + user + "\",\"msg\":\"" + msg + "\"}";

# I did this in case any messages have newline characters (so I won't necessarily store comments as one line the posts file)
def read_simple_json(json_str, start_idx):
    return_dict = {}
    in_quotes = False
    in_esc = False
    quote_start = None
    param = None
    seeker = start_idx
    length = len(json_str)
    if start_idx >= length or json_str[start_idx] != "{":
        return None
    while seeker < length:
        cur_char = json_str[seeker]
        if not in_quotes and not in_esc and cur_char == "}":
            break
        elif in_esc:
            in_esc = False
        elif cur_char == "\\":
            in_esc = True
        # handle quotes
        elif cur_char == "\"":
            if not in_quotes:
                quote_start = seeker
            if in_quotes:
                if param == None and seeker+1 < length and json_str[seeker+1] == ":":
                    param = json_str[quote_start+1:seeker]
                else:
                    return_dict[param] = json_str[quote_start+1:seeker]
                    param = None
                quote_start = None
            in_quotes = not in_quotes
        
        seeker += 1
    # end of while loop
    return_dict['_length'] = seeker+1 - start_idx # how many chars to seek past in order to go past this json object
    return return_dict

# https://wiki.python.org/moin/BaseHttpServer
class MyHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        print("")
        print("GET for", self.client_address)
        print("Path:", self.path)

        output_txt = ""
        data_dict = {}
        try:
            # params is everything after "?"
            params = self.path[self.path.index("?")+1:]
            data_dict = urllib.parse.parse_qs(params)
        finally:
            print("Request params:", data_dict)

        # If no user specified, get all comments.
        if not 'user' in data_dict:
            fd = open(posts_file, "r")
            output_txt = fd.read()
            fd.close()
        # Fetch comments from specified user
        else:
            comment = ""
            cnt = 0
            fd = open(posts_file, "r")
            while True:
                comment = fd.readline()
                if not comment:
                    break
                cmt_json = read_simple_json(comment, 0)
                print(cmt_json)
                if(cmt_json['user'] == data_dict['user'][0]):
                    cnt += 1
                    output_txt += comment
            fd.close()
            print("Number of matches:", cnt)
            if(cnt == 0):
                output_txt = "This user has no comments."

        self.send_response(200, message="Ok")
        self.send_header("Content-Type", "text/plain")
        self.end_headers()
        self.wfile.write(output_txt.encode('utf-8'))
        #self.wfile.write(jsonify("John", "Response").encode('utf-8'))
        

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

        #write to file
        fd = open(posts_file, "a")
        try:
            fd.write(jsonify(user, msg) + "\n")
        finally:
            fd.close()

        self.send_response(200, message="Ok")
        self.send_header("Content-Type", "text/plain")
        self.end_headers()
        self.wfile.write(("Message Posted:\n" + jsonify(user, msg)).encode('utf-8'))


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
