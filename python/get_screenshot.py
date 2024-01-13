from pathlib import Path
import pygetwindow as gw
from PIL import ImageGrab
import pyautogui
import win32gui
import win32ui
import win32con
from ctypes import windll
import win32api
from PIL import Image


#base_path = Path(r"I:\ZyngaData")
# base_path = Path(r"D:\ZyngaData")
base_path = Path(r"\\wsl.localhost\Ubuntu-20.04\home\eric\git\poker_eval\python\datasets\incoming")

def find_title()->str:

    zynga_title = None 
    titles = gw.getAllTitles()
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
    
    return zynga_title


def get_file_path()->Path:
    file_path  = None
    for i in range(0, 1000):
        file_path = base_path / f"zynga_{i}.bmp"
        png_path = base_path / f"zynga_{i}.png"
        if not png_path.exists():
            break

    if file_path is None:
        raise Exception("Could not find free filename")
    return file_path


def get_screenshot():
    

    zynga_title = find_title() 

    # Free filename
    file_path  = get_file_path()

    print(f"Fetching window title [{zynga_title}]")
    
    
    # Find the window by its title
    hwnd = win32gui.FindWindow(None, zynga_title)
    if hwnd == 0:
        raise Exception('Window not found: ' + zynga_title)

    left, top, right, bot = win32gui.GetWindowRect(hwnd)
    w = right - left
    h = bot - top
    print(f"Window dims: Width: {w}, Height: {h}")
    print(f"Left: {left}, Top: {top}, Right: {right}, Bot: {bot}")

    if True:
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


    if file_path.exists():
        file_path.unlink()
    
    

# i:\python\scripts\pip install pygetwindow

# i:\python\python.exe "\\wsl.localhost\Ubuntu-22.04\home\eric\git\poker_eval\python\get_screenshot.py"

if __name__ == "__main__":
    get_screenshot()


