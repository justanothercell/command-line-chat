import websocket
import _thread
import time
import rel

def on_message(ws, message):
    print('message:', message)

def on_error(ws, error):
    print('error:', error)

def on_close(ws, close_status_code, close_msg):
    print('### closed ###')

def on_open(ws):
    print(ws.__dict__)
    print('Opened connection')

if __name__ == '__main__':
    websocket.enableTrace(True)
    ws = websocket.WebSocketApp('wss://clc.onrender.com/ws/3d4ffd31c28549c992dc6f25ce1c390a',
                                on_open=on_open,
                                on_message=on_message,
                                on_error=on_error,
                                on_close=on_close)

    ws.run_forever(dispatcher=rel)  # Set dispatcher to automatic reconnection
    rel.signal(2, rel.abort)  # Keyboard Interrupt
    rel.dispatch()