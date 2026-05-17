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

def get_categories():
    resp = requests.get(f'{BASE_URL}/api/config/items?config_type=account_category&page=1&page_size=100', headers=HEADERS)
    items = resp.json()['items']
    stock_id = next(item['id'] for item in items if item['name'] == '股票')
    wealth_id = next(item['id'] for item in items if item['name'] == '理财')
    return stock_id, wealth_id

def get_investment(investment_type, keyword):
    resp = requests.get(f'{BASE_URL}/api/investments?user_id={USER_ID}&investment_type={investment_type}&show_sold=true&keyword={keyword}', headers=HEADERS)
    return resp.json()['items'][0] if resp.json()['items'] else None

def smoke_test():
    stock_cat_id, wealth_cat_id = get_categories()
    timestamp = int(time.time())
    
    types = [
        ('stock', stock_cat_id, 'STK'),
        ('wealth', wealth_cat_id, 'WTH')
    ]
    
    results = {}

    for investment_type, cat_id, prefix in types:
        code = f'{prefix}{timestamp}'
        type_results = []
        
        # 1. Open Position
        payload = {
            'bill_type': 'investment',
            'investment_type': investment_type,
            'action': 'open_position',
            'user_id': USER_ID,
            'category_id': cat_id,
            'amount': 1000,
            'share_amount': 100,
            'product_name': f'Test Product {code}',
            'product_code': code,
            'organization_name': 'Test Org',
            'occurred_at': f'{time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())}'
        }
        resp = requests.post(f'{BASE_URL}/api/bills', headers=HEADERS, json=payload)
        type_results.append({'step': 'open', 'success': resp.status_code == 201})
        inv = get_investment(investment_type, code)
        type_results[-1]['metrics'] = {'shares': float(inv['total_share']), 'cost': float(inv['total_cost'])}
        related_id = inv['id']

        # 2. Add Position
        payload = {
            'bill_type': 'investment',
            'investment_type': investment_type,
            'action': 'add_position',
            'user_id': USER_ID,
            'related_investment_id': related_id,
            'amount': 500,
            'share_amount': 50,
            'occurred_at': f'{time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())}'
        }
        resp = requests.post(f'{BASE_URL}/api/bills', headers=HEADERS, json=payload)
        type_results.append({'step': 'add', 'success': resp.status_code == 201})
        inv = get_investment(investment_type, code)
        type_results[-1]['metrics'] = {'shares': float(inv['total_share']), 'cost': float(inv['total_cost'])}

        # 3. Reduce Position
        payload = {
            'bill_type': 'investment',
            'investment_type': investment_type,
            'action': 'reduce_position',
            'user_id': USER_ID,
            'related_investment_id': related_id,
            'amount': 260,
            'share_amount': 20,
            'occurred_at': f'{time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())}'
        }
        resp = requests.post(f'{BASE_URL}/api/bills', headers=HEADERS, json=payload)
        type_results.append({'step': 'reduce', 'success': resp.status_code == 201})
        inv = get_investment(investment_type, code)
        type_results[-1]['metrics'] = {'shares': float(inv['total_share']), 'cost': float(inv['total_cost'])}

        # 4. Oversell
        payload = {
            'bill_type': 'investment',
            'investment_type': investment_type,
            'action': 'reduce_position',
            'user_id': USER_ID,
            'related_investment_id': related_id,
            'amount': 1,
            'share_amount': 9999,
            'occurred_at': f'{time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())}'
        }
        resp = requests.post(f'{BASE_URL}/api/bills', headers=HEADERS, json=payload)
        type_results.append({'step': 'oversell', 'success': resp.status_code != 201})
        
        results[investment_type] = type_results

    print(json.dumps(results, indent=2))

if __name__ == '__main__':
    smoke_test()
