import requests
import json
import time
import base64
import hashlib

def generate_token(user_id, secret, ttl_minutes=60):
    expires_at = int(time.time()) + (ttl_minutes * 60)
    msg = f'{user_id}:{expires_at}:{secret}'.encode()
    signature = hashlib.sha256(msg).hexdigest()
    token_str = f'{user_id}:{expires_at}:{signature}'
    return base64.urlsafe_b64encode(token_str.encode()).decode().rstrip('=')

BASE_URL = 'http://127.0.0.1:8080'
SECRET = 'change-me-in-production'
USER_ID = 17
TOKEN = generate_token(USER_ID, SECRET)
HEADERS = {'Authorization': f'Bearer {TOKEN}', 'Content-Type': 'application/json'}

current_month_start = time.strftime("%Y-%m-01")
current_month_end = time.strftime("%Y-%m-31") # Rough end of month for simple test
url = f'{BASE_URL}/api/dashboard/summary?start_date={current_month_start}&end_date={current_month_end}&user_id={USER_ID}'

try:
    resp = requests.get(url, headers=HEADERS)
    print(f'STATUS_CODE:{resp.status_code}')
    print(f'RESPONSE_BODY:{resp.text}')
except Exception as e:
    print(f'ERROR:{e}')