import socket
import sys

servport = 1234
proxyport = 1111

def server(port): 
    print('starting server on port {}'.format(port))
    address = ('', port)
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)

    # allow reuse of socket addresses for faster testing
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)

    sock.setsockopt(socket.SOL_IP, socket.IP_TRANSPARENT, 1)

    sock.bind(address)
    sock.listen(1)

    try:
        while True:
            conn, addr = sock.accept()
            print('connection received from {}'.format(addr))
            print('connection destined to {}'.format(conn.getsockname()))
            try:
                while True:
                    data = conn.recv(16)
                    if data:
                        print('recv: {}'.format(data.decode('utf-8')))
                        conn.sendall(data)
                    else:
                        print('connection ended')
                        break
            finally:
                conn.close()
    except KeyboardInterrupt:
        print('stopping server')
    finally:
        sock.close()

if __name__ == '__main__':
    if (len(sys.argv) != 2):
        print('usage: echoclient.py [--server|--proxy]')
        sys.exit(1)

    if (sys.argv[1] == '--proxy'):
        port = proxyport
    elif (sys.argv[1] == '--server'):
        port = servport
    else:
        print('unknown argument: {}'.format(sys.argv[1]))
        sys.exit(1)

    server(port)
