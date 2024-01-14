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

import pytz  # Import the pytz library for timezone handling

# Create a timezone object for GMT/UTC
gmt_timezone = pytz.timezone('UTC')

#base_path = Path(r"\\wsl.localhost\Ubuntu-20.04\home\eric\git\poker_eval\python\datasets\incoming")
work_path = Path(os.environ["WORK_PATH"])
incoming_path = Path(os.environ["INCOMING_PATH"])

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
    print(f"Left: {left}, Top: {top}, Right: {right}, Bot: {bot}")

    if False:
        win32gui.SetForegroundWindow(hwnd)
        x, y, x1, y1 = win32gui.GetClientRect(hwnd)
        x, y = win32gui.ClientToScreen(hwnd, (x, y))
        x1, y1 = win32gui.ClientToScreen(hwnd, (x1 - x, y1 - y))
        img = pyautogui.screenshot(region=(x, y, x1, y1))
        
    else:
        hwin = win32gui.GetDesktopWindow()

        desktop_dc = win32gui.GetWindowDC(hwin)

        srcdc = win32ui.CreateDCFromHandle(desktop_dc)
        memdc = srcdc.CreateCompatibleDC()
        bmp = win32ui.CreateBitmap()
        bmp.CreateCompatibleBitmap(srcdc, w, h)
        memdc.SelectObject(bmp)
        memdc.BitBlt((0, 0), (w, h), srcdc, (left, top), win32con.SRCCOPY)
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
    target_path = incoming_path / png_file_path.name
    print(f"Moving PNG to file [{target_path}]")
    shutil.move(png_file_path, target_path)
    
    

if __name__ == "__main__":
    for i in range(0, 10_000):
        get_screenshot()
        time.sleep(0.75)
