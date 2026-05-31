import os
os.environ['PLAYWRIGHT_BROWSERS_PATH'] = '/tmp/playwright-browsers'
from playwright.sync_api import sync_playwright
with sync_playwright() as p:
    b = p.chromium.launch(headless=True, args=['--no-sandbox'])
    print('CHROMIUM OK from /tmp')
    b.close()
