import socket
import sys
import requests
import forum_util
from requests.exceptions import HTTPError

ser_port = 9090
ser_ip = "172.40.17.19" # server-router external-facing ip
# if you want direct connection to server (no server proxy), try 172.40.17.10
default_cert = "/usr/local/share/ca-certificates/forum.crt"

class conn_info:
    def __init__(self, ip, port, mode, cert):
        self.ip = ip
        self.port = port
        self.mode = mode # http, https, etc.
        self.cert = cert


def get(conn):
    mode = conn.mode
    URL = mode.lower() + "://" + conn.ip + ":" + str(conn.port) + "/"
    user = input("Whose messages do you want to display? (Leave blank to display all messages)\n")
    PARAMS = {'user':user}
    try:
        r = requests.get(url = URL, params = PARAMS, verify = conn.cert) # conn.cert can be False or filename of cert
    except Exception as e:
        print(e)
        raise Exception("Error connecting to server")
    r.encoding = 'utf-8' #tells how to decode response into string
    print("Comments:")
    try:
        r.raise_for_status()
        r_json = r.json()
        for post in r_json:
            print("________________________________")
            print("User:", post['user'])
            print(post['msg'])
    except HTTPError as http_err:
        print(http_err)
    except:
        print("Could not parse response as JSON")
        print(r.text)

def post(conn):
    mode = conn.mode
    URL = mode.lower() + "://" + conn.ip + ":" + str(conn.port) + "/"
    user = input("Enter a username to post a comment: ")
    comment = input("Enter a comment to post: ")
    DATA = {'user': user, 'msg': comment}
    try:
        r = requests.post(url = URL, data = DATA, verify = conn.cert)
    except Exception as e:
        print(e)
        raise Exception("Error connecting to server")
    r.encoding = 'utf-8' #tells how to decode response into string
    try:
        r.raise_for_status() # throw an error if status is not within 200-400
        print(r.text)
    except HTTPError as http_err:
        print(http_err)

# https://www.geeksforgeeks.org/get-post-requests-using-python/
def client(conn):
    print("Starting {} client on port {}".format(mode, port))

    try:
        while True:
            action = input("Do you want to get, post, or quit? ")
            if (action.upper() == "GET"):
                keep_going = get(conn)
            elif (action.upper() == "POST"):
                keep_going = post(conn)
            elif (action.upper() == "QUIT"):
                break
            print("")
    except KeyboardInterrupt:
        print("\nkeyboard interrupt")
    except Exception as e:
        print("\nProgram terminated due to error:")
        print("  ", e)
    print('Stopping Client')
    print("Goodbye!")
    return


if __name__ == '__main__':
    ip = ser_ip
    port = ser_port
    mode = "HTTP"
    cert = False
    if len(sys.argv) == 2:
        ip = sys.argv[1] # legacy implementation
    elif len(sys.argv) == 1:
        print("Usage: python3 example_client.py " +
            "[--ip default server-proxy " + ser_ip + "] " +
            "[--port default " + str(ser_port) + "] " +
            "[--mode https or default http] \n")
    else:
        for i in range(len(sys.argv)-1):
            arg = sys.argv[i]
            next_arg = sys.argv[i+1]
            if (arg == "--ip"):
                ip = next_arg
            elif (arg == "--port"):
                port = int(next_arg)
            elif (arg == "--mode"):
                mode = next_arg.upper()
    
    if mode == "HTTPS":
        #raise Exception("HTTPS client is not supported yet")
        tf = input("\nDo you want to require certificate verification? (T/F) ")
        cert = not (tf.upper() == "F") # default to true for unrecognized input, otherwise set False
        if cert:
            print("Default cert is", default_cert)
            inp = input("Which certificate file do you want to use? (Leave blank for default)\n")
            if inp == "":
                cert = default_cert
            else:
                cert = inp
    elif mode != "HTTP":
        raise Exception("Client mode {} is not supported.".format(mode))
    conn = conn_info(ip, port, mode, cert) # will either be false or the name of a certificate file or chain
    client(conn)
