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
    print('Opened connection')

if __name__ == '__main__':
    websocket.enableTrace(True)
    ws = websocket.WebSocketApp('ws://127.0.0.1:8000/ws/f6072768257140e7bca814330a84958b',
                                on_open=on_open,
                                on_message=on_message,
                                on_error=on_error,
                                on_close=on_close)

    ws.run_forever(dispatcher=rel)  # Set dispatcher to automatic reconnection
    rel.signal(2, rel.abort)  # Keyboard Interrupt
    rel.dispatch()