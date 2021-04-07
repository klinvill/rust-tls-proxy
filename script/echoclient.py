import socket
import sys

ser_port = 1234
cli_port = 4321

def client(ip):
    server_address = (ip, ser_port)
    sock = socket.socket(
        socket.AF_INET, socket.SOCK_STREAM, socket.IPPROTO_TCP
    )
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.bind(('', cli_port))
    print('connecting to {}'.format(server_address))

    try:
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
    if len(sys.argv) != 2:
        ip = "172.40.17.10"
    else:
        ip = sys.argv[1]

    client(ip)
