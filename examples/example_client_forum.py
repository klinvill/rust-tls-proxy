import socket
import sys
import requests

ser_port = 9443 # server-router using 9443 instead of 443 to avoid sudo during test

def jsonify(user, msg):
    max_chars = 256
    num_attributes = 2
    max_chars -= (num_attributes*6+1) # "":"", and {} but last line has no comma
    return "{\"user\":\"" + user + "\",\"msg\":\"" + msg + "\"}";

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
    print(r.text)
    #print("Now printing JSON")
    #print(r.json())
    return continue_prompt()

def post(addr):
    URL = "http://" + addr[0] + ":" + str(addr[1]) + "/"
    user = input("Enter a username to post a comment: ")
    comment = input("Enter a comment to post: ")
    DATA = {'user': user, 'msg': comment}
    r = requests.post(url = URL, data = DATA)
    resp = r.text
    print(resp)
    '''
    print("Received:\n" + get-post-requests-using-python)

    try:
        json_start = resp.index("{")
        resp_start = resp[0:json_start]
        com = r.json()
        print(com)
    '''
    return continue_prompt()

# https://www.geeksforgeeks.org/get-post-requests-using-python/
def client(ip):
    server_address = (ip, ser_port)
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

if __name__ == '__main__':
    if len(sys.argv) != 2:
        ip = "172.40.17.19" # server-router external-facing ip
        for i in range(len(sys.argv)-1):
            arg = sys.argv[i]
            if (arg == "--ip" and sys.argv[i+1] != "default"):
                ip = sys.argv[i+1]
            elif (arg == "--port" and sys.argv[i+1] != "default"):
                ser_port = int(sys.argv[i+1])
    else:
        ip = sys.argv[1]

    client(ip)
