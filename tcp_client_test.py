import socket

# サーバーのホストとポート番号を設定
server_host = '127.0.0.1'  # サーバーのホスト名またはIPアドレス
server_port = 8990  # サーバーのポート番号

# サーバーに接続
client_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
client_socket.connect((server_host, server_port))

try:
    while True:
        # キーボードからテキストを入力
        message = input("メッセージを入力してください (終了するには 'exit' と入力): ")

        # 'exit'と入力されたらクライアントを終了
        if message == 'exit':
            break

        # メッセージをサーバーに送信
        client_socket.sendall(message.encode('utf-8'))

        # サーバーからの応答を受信
        response = client_socket.recv(1024).decode('utf-8')
        print("サーバーからの応答:", response)

except KeyboardInterrupt:
    print("ユーザーによって中断されました.")

finally:
    # クライアントソケットを閉じる
    client_socket.close()
