import socket
import sys
import requests
import forum_util
from requests.exceptions import HTTPError

ser_port = 9090

def get(addr, mode):
    if(mode == "HTTPS"):
        raise Exception("HTTPS GET is not supported yet")
    URL = "http://" + addr[0] + ":" + str(addr[1]) + "/"
    user = input("Whose messages do you want to display? (Leave blank to display all messages)\n")
    PARAMS = {'user':user}
    try:
        r = requests.get(url = URL, params = PARAMS)
    except:
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

def post(addr, mode):
    if(mode == "HTTPS"):
        raise Exception("HTTPS POST is not supported yet")
    URL = "http://" + addr[0] + ":" + str(addr[1]) + "/"
    user = input("Enter a username to post a comment: ")
    comment = input("Enter a comment to post: ")
    DATA = {'user': user, 'msg': comment}
    try:
        r = requests.post(url = URL, data = DATA)
    except:
        raise Exception("Error connecting to server")
    r.encoding = 'utf-8' #tells how to decode response into string
    try:
        r.raise_for_status() # throw an error if status is not within 200-400
        print(r.text)
    # https://realpython.com/python-requests/
    except HTTPError as http_err:
        print(http_err)

# https://www.geeksforgeeks.org/get-post-requests-using-python/
def client(ip, port, mode):
    # so far it seems like all I need to do is change the http:// to https:// in the get and post methods. I'm not sure though.
    if(mode == "HTTPS"):
        raise Exception("HTTPS client is not supported yet")
    server_address = (ip, port)
    try:
        while True:
            action = input("Do you want to get, post, or quit? ")
            if (action.upper() == "GET"):
                keep_going = get(server_address, mode)
            elif (action.upper() == "POST"):
                keep_going = post(server_address, mode)
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
    port = ser_port
    mode = "HTTP"
    if len(sys.argv) == 2:
        ip = sys.argv[1] # legacy implementation
    else:
        ip = "172.40.17.19" # server-router external-facing ip
        for i in range(len(sys.argv)-1):
            arg = sys.argv[i]
            if (arg == "--ip" and sys.argv[i+1] != "default"):
                ip = sys.argv[i+1]
            elif (arg == "--port" and sys.argv[i+1] != "default"):
                port = int(sys.argv[i+1])
            elif (arg == "--mode"):
                mode = (sys.argv[i+1]).upper()
        
    if mode == "HTTP" or mode == "HTTPS":
        client(ip, port, mode)
    else:
        raise Exception("Client mode {} is not supported.".format(mode))
