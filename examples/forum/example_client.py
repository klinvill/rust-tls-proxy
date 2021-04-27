import socket
import sys
import requests
import forum_util
from requests.exceptions import HTTPError

ser_port = 9090
ser_ip = "172.40.17.19" # server-router external-facing ip
# if you want direct connection to server (no server proxy), try 172.40.17.10

class conn_info:
    def __init__(self, ip, port, mode, secure):
        self.ip = ip
        self.port = port
        self.mode = mode # http, https, etc.
        self.secure = secure


def get(conn):
    mode = conn.mode
    URL = mode.lower() + "://" + conn.ip + ":" + str(conn.port) + "/"
    user = input("Whose messages do you want to display? (Leave blank to display all messages)\n")
    PARAMS = {'user':user}
    try:
        r = requests.get(url = URL, params = PARAMS, verify = conn.secure)
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
        r = requests.post(url = URL, data = DATA, verify = conn.secure)
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
    # so far it seems like all I need to do is change the http:// to https:// in the get and post methods. I'm not sure though.
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
    secure = False
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
        
    if mode == "HTTP":
        print("Starting HTTP client on port {}".format(port))
    elif mode == "HTTPS":
        raise Exception("HTTPS client is not supported yet")
        print("Starting HTTPS client on port {}".format(port))
        tf = input("\nDo you want to require certificate verification? (T/F) ")
        secure = not (tf.upper() == "F") # default to true for unrecognized input, otherwise set False
    else:
        raise Exception("Client mode {} is not supported.".format(mode))
    conn = conn_info(ip, port, mode, secure)
    client(conn)
