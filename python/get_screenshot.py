from pathlib import Path
import shutil
import time
import pygetwindow as gw
from PIL import ImageGrab
import pyautogui
import win32gui
import win32ui
import win32con
from ctypes import windll
import win32api
from PIL import Image
from datetime import datetime
import os
import socket
import pytz  # Import the pytz library for timezone handling
import io
from PIL import Image

# Create a timezone object for GMT/UTC
gmt_timezone = pytz.timezone('UTC')

datasets_path = Path(os.environ["DATASETS_PATH"])
work_path = datasets_path / "work"
incoming_path = datasets_path 
save_path = datasets_path / "save"

save_mode = int(os.environ["SAVE_MODE"]) == 1

if not work_path.exists():
    raise Exception(f"Work path [{work_path}] does not exist")

# Anaconda installation
# Admin prompt
# i:\python\Scripts\pip install pyautogui
# i:\python\scripts\pip install pygetwindow
# \\wsl.localhost\Ubuntu-22.04\home\eric\git\poker_eval\dev\get_ss.bat


title_to_find = os.environ["WINDOW_TITLE_TO_FIND"]


def find_title()->str:

    full_title = None 
    titles = gw.getAllTitles()
    for t in titles:
        print(f"Lookign at title [{t}]")
        if title_to_find in t:
            full_title = t
            break
        

    if full_title is None:
        raise Exception("window not found")
    
    return full_title


def get_file_path()->Path:
    file_path  = None

    # get GMT time yyyyMMDD_HHmmss_mmm

    now = datetime.now(gmt_timezone)

    formatted_string = now.strftime("%Y%m%d_%H%M%S_%f")[:-3]
    
    file_path = work_path / f"{formatted_string}.bmp"
    
    return file_path


def get_screenshot():
    

    window_title = find_title() 

    # Free filename
    file_path  = get_file_path()

    print(f"Fetching window title [{window_title}]")
    
    
    # Find the window by its title
    hwnd = win32gui.FindWindow(None, window_title)
    if hwnd == 0:
        raise Exception('Window not found: ' + window_title)

    left, top, right, bot = win32gui.GetWindowRect(hwnd)
    w = right - left
    h = bot - top
    print(f"Window dims: Width: {w}, Height: {h}")
    print(f"Left: {left}, Top: {top}, Right: {right}, Bottom: {bot}")

    v_width = win32api.GetSystemMetrics(win32con.SM_CXVIRTUALSCREEN)
    v_height = win32api.GetSystemMetrics(win32con.SM_CYVIRTUALSCREEN)
    v_left = win32api.GetSystemMetrics(win32con.SM_XVIRTUALSCREEN)
    v_top = win32api.GetSystemMetrics(win32con.SM_YVIRTUALSCREEN)

    print(f"Virtual dims: Width: {v_width}, Height: {v_height}")
    print(f"Virtual Left: {v_left}, Virtual Top: {v_top}")

    hwin = win32gui.GetDesktopWindow()
    desktop_dc = win32gui.GetWindowDC(hwin)
    srcdc = win32ui.CreateDCFromHandle(desktop_dc)

    # Get the scaling factor
    LOGPIXELSX = 88
    LOGPIXELSY = 90
    actual_dpi_x = srcdc.GetDeviceCaps(LOGPIXELSX)
    actual_dpi_y = srcdc.GetDeviceCaps(LOGPIXELSY)
    scale_factor_x = actual_dpi_x / 96.0
    scale_factor_y = actual_dpi_y / 96.0
    print(f"Scale factor: X: {scale_factor_x}, Y: {scale_factor_y}")

    memdc = srcdc.CreateCompatibleDC()
    bmp = win32ui.CreateBitmap()
    bmp.CreateCompatibleBitmap(srcdc, int(w / scale_factor_x), int(h / scale_factor_y))
    memdc.SelectObject(bmp)
    memdc.BitBlt((0, 0), (int(w / scale_factor_x), int(h / scale_factor_y)), srcdc, (
        int(left / scale_factor_x), int(top / scale_factor_y)), win32con.SRCCOPY)
    print(f"Saving to file [{file_path}]")
    bmp.SaveBitmapFile(memdc, str(file_path))


    win32gui.DeleteObject(bmp.GetHandle())
    memdc.DeleteDC()
    srcdc.DeleteDC()
    win32gui.ReleaseDC(hwin, desktop_dc)

    img = Image.open(file_path)

    png_file_path = file_path.with_suffix('.png')  # Change file extension to .png
    print(f"Saving PNG to file [{png_file_path}]")
    img.save(png_file_path, 'PNG')

    # delete the bmp
    if file_path.exists():
        file_path.unlink()

    # move the png to incoming
    if save_mode:
        target_path = save_path / png_file_path.name
    else:
        target_path = incoming_path / png_file_path.name
    print(f"Moving PNG to file [{target_path}]")
    shutil.move(png_file_path, target_path)


def get_screenshot_to_buffer():
    

    window_title = find_title() 

    print(f"Fetching window title [{window_title}]")
    
    
    # Find the window by its title
    hwnd = win32gui.FindWindow(None, window_title)
    if hwnd == 0:
        raise Exception('Window not found: ' + window_title)

    left, top, right, bot = win32gui.GetWindowRect(hwnd)
    w = right - left
    h = bot - top
    print(f"Window dims: Width: {w}, Height: {h}")
    print(f"Left: {left}, Top: {top}, Right: {right}, Bot: {bot}")

    hwin = win32gui.GetDesktopWindow()

    desktop_dc = win32gui.GetWindowDC(hwin)

    srcdc = win32ui.CreateDCFromHandle(desktop_dc)
    memdc = srcdc.CreateCompatibleDC()
    bmp = win32ui.CreateBitmap()
    bmp.CreateCompatibleBitmap(srcdc, w, h)
    memdc.SelectObject(bmp)
    memdc.BitBlt((0, 0), (w, h), srcdc, (left, top), win32con.SRCCOPY)
        
    bmpinfo = bmp.GetInfo()
    bmpstr = bmp.GetBitmapBits(True)
    im = Image.frombuffer(
        'RGB',
        (bmpinfo['bmWidth'], bmpinfo['bmHeight']),
        bmpstr, 'raw', 'BGRX', 0, 1)

    # Create an in-memory byte stream
    output = io.BytesIO()
    im.save(output, format='PNG')

    win32gui.DeleteObject(bmp.GetHandle())
    memdc.DeleteDC()
    srcdc.DeleteDC()
    win32gui.ReleaseDC(hwin, desktop_dc)

    byte_data = output.getvalue()
    output.close()

    return byte_data
    
def listen():
    # TCP server settings
    host = '0.0.0.0'  # Listen on all interfaces
    port = 4242       # Port number

    # Create a socket
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind((host, port))
        s.listen()

        print(f"Listening on {host}:{port}")

        while True:
            conn, addr = s.accept()
            with conn:
                print(f"Connected by {addr}")
                while True:
                    data = conn.recv(1024)
                    if not data:
                        break
                    if data == b'1':
                        screenshot = get_screenshot_to_buffer()
                        conn.sendall(screenshot)
                        conn.close()
                        break


if __name__ == "__main__":
    if save_mode:
        get_screenshot()
    else:
        listen()
