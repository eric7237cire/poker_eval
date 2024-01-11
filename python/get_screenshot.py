from pathlib import Path
import pygetwindow as gw
from PIL import ImageGrab

import pygetwindow as gw
import win32gui
import win32ui
import win32con
from ctypes import windll
import win32api
from PIL import Image


base_path = Path(r"I:\ZyngaData")
def get_screenshot():
    titles = gw.getAllTitles()

    zynga_title = None 



    for t in titles:
        print(f"Lookign at title [{t}]")
        if "Zynga Poker" in t:
            zynga_title = t
            break
        if "redb rust - Brave Search - Brave" in t:
            zynga_title = t
            break

    if zynga_title is None:
        raise Exception("Zynga Poker window not found")
    
    # Free filename
    for i in range(0, 1000):
        file_path = base_path / f"zynga_{i}.png"
        if not file_path.exists():
            break

    print(f"Fetching window title [{zynga_title}]")
    # window = gw.getWindowsWithTitle(zynga_title)

    # if len(window) == 0:
    #     print(f"Window not found [{zynga_title}]]")
    #     return
    
    # Find the window by its title
    hwnd = win32gui.FindWindow(None, zynga_title)
    if hwnd == 0:
        raise Exception('Window not found: ' + zynga_title)

    left, top, right, bot = win32gui.GetWindowRect(hwnd)
    w = right - left
    h = bot - top
    print(f"Window dims: Width: {w}, Height: {h}")
    print(f"Left: {left}, Top: {top}, Right: {right}, Bot: {bot}")

    hwin = win32gui.GetDesktopWindow()
    # v_width = win32api.GetSystemMetrics(win32con.SM_CXVIRTUALSCREEN)
    # v_height = win32api.GetSystemMetrics(win32con.SM_CYVIRTUALSCREEN)
    # v_left = win32api.GetSystemMetrics(win32con.SM_XVIRTUALSCREEN)
    # v_top = win32api.GetSystemMetrics(win32con.SM_YVIRTUALSCREEN)

    desktop_dc = win32gui.GetWindowDC(hwin)

    srcdc = win32ui.CreateDCFromHandle(desktop_dc)
    memdc = srcdc.CreateCompatibleDC()
    bmp = win32ui.CreateBitmap()
    bmp.CreateCompatibleBitmap(srcdc, w, h)
    memdc.SelectObject(bmp)
    memdc.BitBlt((0, 0), (w, h), srcdc, (left, top), win32con.SRCCOPY)
    bmp.SaveBitmapFile(memdc, str(file_path))

    

    # im = Image.frombuffer(
    #     'RGB',
    #     (bmpinfo['bmWidth'], bmpinfo['bmHeight']),
    #     bmpstr, 'raw', 'BGRX', 0, 1)
    
    
    # if result == 1:
    #     #PrintWindow Succeeded
    #     im.save(file_path)
    # else:
    #     #PrintWindow Failed
    #     raise Exception("Failed to get screenshot of window")

    
    win32gui.ReleaseDC(hwnd, desktop_dc)

    
    

# i:\python\scripts\pip install pygetwindow

# i:\python\python.exe "\\wsl.localhost\Ubuntu-22.04\home\eric\git\poker_eval\python\get_screenshot.py"

if __name__ == "__main__":
    get_screenshot()


