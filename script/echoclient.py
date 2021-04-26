import socket
import sys

# using port numbers prepended with 9s to avoid calling sudo during testing
http_port = 9980
https_port = 9443
redir_port = 8080

def client(ip):
    server_address = (ip, http_port)
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
            recv_str = sock.recv(16).decode('utf-8')

            if not recv_str:
                print("server closed connection")
                return
            else:
                print(recv_str)

    except KeyboardInterrupt:
        print('stopping client')
    finally:
        sock.close()

if __name__ == '__main__':
    if len(sys.argv) != 2:
        ip = "172.40.17.19" # server-router external-facing ip
    else:
        ip = sys.argv[1]

    client(ip)
