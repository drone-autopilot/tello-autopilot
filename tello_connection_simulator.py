import threading
import time
import socket

HOST = "127.0.0.1"
STATE_PORT = 8890
CMD_PORT = 8889
VIDEO_PORT = 11111
BUF_SIZE = 1024

def state_server():
    # print("start state server")

    # sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    # sock.bind((HOST, STATE_PORT))
    # _, cli_addr = sock.recvfrom(BUF_SIZE)

    # while True:
    #     sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    #     sock.bind((HOST, STATE_PORT))

    #     try:
    #         sock.sendto(b"pitch:0;roll:2;yaw:0;vgx:0;vgy:0;vgz:0;templ:83;temph:85;tof:6553;h:0;bat:83;baro:193.06;time:0;agx:-5.00;agy:-48.00;agz:-998.00;\r\n", cli_addr)
    #         time.sleep(1)

    #     except KeyboardInterrupt:
    #         sock.close()
    #         break
    pass

def cmd_server():
    print("start cmd server")

    while True:
        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        sock.bind((HOST, CMD_PORT))

        try:
            msg, cli_addr = sock.recvfrom(BUF_SIZE)
            msg = msg.decode()
            print(f"received \"{msg}\" from {cli_addr}")
            time.sleep(1)

            sock.sendto(b"ok", cli_addr)

        except KeyboardInterrupt:
            sock.close()
            break

def video_server():
    # print("start video server")

    # sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    # sock.bind((HOST, VIDEO_PORT))
    # _, cli_addr = sock.recvfrom(BUF_SIZE)

    # while True:
    #     sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    #     sock.bind((HOST, VIDEO_PORT))

    #     try:
    #         sock.sendto(b"deadbeaf", cli_addr)
    #         time.sleep(1)

    #     except KeyboardInterrupt:
    #         sock.close()
    #         break
    pass


if __name__ == "__main__":
    s = threading.Thread(target=state_server)
    c = threading.Thread(target=cmd_server)
    v = threading.Thread(target=video_server)

    s.start()
    c.start()
    v.start()

    s.join()
    c.join()
    v.join()
