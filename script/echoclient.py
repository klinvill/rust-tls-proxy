import socket
import sys

ser_port = 1234

def client(ip):
    server_address = (ip, ser_port)
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)

    try:
        print('connecting to {}'.format(server_address))
        sock.connect(server_address)
    except ConnectionRefusedError:
        print('connection to {} refused'.format(server_address))
        return

    try:
        while True:
            in_str = input(">>")
            sock.sendall(in_str.encode('utf-8'))
            print(sock.recv(16).decode('utf-8'))
    except KeyboardInterrupt:
        print('stopping client')
    finally:
        sock.close()

if __name__ == '__main__':
    # 2 arugments = change target ip, otherwise check for option flags
    if len(sys.argv) != 2:
        ip = "172.40.17.19" # server-router external-facing ip
        for i in range(len(sys.argv)-1):
            arg = sys.argv[i]
            if (arg == "--ip"):
                ip = sys.argv[i+1]
            elif (arg == "--port"):
                ser_port = int(sys.argv[i+1])
    else:
        ip = sys.argv[1]

    client(ip)
