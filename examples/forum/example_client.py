import socket
import sys
import requests
import forum_util

ser_port = 9090

def continue_prompt():
    ctn = input("Continue? (T/F)")
    if ctn.upper() == "T":
        ctn = True
    else:
        ctn = False
    return ctn

def get(addr):
    URL = "http://" + addr[0] + ":" + str(addr[1]) + "/"
    user = input("Whose messages do you want to display? (Leave blank to display all messages)\n")
    PARAMS = {'user':user}
    r = requests.get(url = URL, params = PARAMS)
    print("Comments:")
    #print(r.text)
    try:
        r_json = r.json()
        for post in r_json:
            print("________________________________")
            print("User:", post['user'])
            print(post['msg'])
    except:
        print("Could not parse response as JSON")
        print(r.text)
    print("")
    return continue_prompt()

def post(addr):
    URL = "http://" + addr[0] + ":" + str(addr[1]) + "/"
    user = input("Enter a username to post a comment: ")
    comment = input("Enter a comment to post: ")
    DATA = {'user': user, 'msg': comment}
    r = requests.post(url = URL, data = DATA)
    resp = r.text
    print(resp)
    return continue_prompt()

# https://www.geeksforgeeks.org/get-post-requests-using-python/
def client(ip, port):
    server_address = (ip, port)
    keep_going = True
    try:
        while keep_going:
            mode = input("Do you want to get or post? ")
            if (mode.upper() == "GET"):
                keep_going = get(server_address)
            elif (mode.upper() == "POST"):
                keep_going = post(server_address)
    except KeyboardInterrupt:
        print('stopping client')
    return


if __name__ == '__main__':
    port = ser_port
    if len(sys.argv) != 2:
        ip = "172.40.17.19" # server-router external-facing ip
        for i in range(len(sys.argv)-1):
            arg = sys.argv[i]
            if (arg == "--ip" and sys.argv[i+1] != "default"):
                ip = sys.argv[i+1]
            elif (arg == "--port" and sys.argv[i+1] != "default"):
                port = int(sys.argv[i+1])
    else:
        ip = sys.argv[1]

    client(ip, port)
