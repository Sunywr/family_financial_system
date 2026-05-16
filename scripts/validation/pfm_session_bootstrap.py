#!/usr/bin/env python3
"""Bootstrap PFM session auth for validation scripts.

This helper supports session-cookie mode environments where captcha is PNG
and cannot be auto-decoded. It performs:
1) GET /api/utils/captcha and saves image to disk.
2) Prompt user to input captcha code manually.
3) POST /api/utils/login.
4) Print Cookie header value for downstream scripts.
"""

from __future__ import annotations

import argparse
import base64
import http.cookiejar
import json
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any


def http_json(
    opener: urllib.request.OpenerDirector,
    base_url: str,
    path: str,
    method: str = "GET",
    body: dict[str, Any] | None = None,
    timeout: int = 20,
) -> dict[str, Any]:
    url = f"{base_url.rstrip('/')}{path}"
    data = None if body is None else json.dumps(body, ensure_ascii=False).encode("utf-8")
    req = urllib.request.Request(url=url, data=data, method=method)
    req.add_header("Accept", "application/json")
    if body is not None:
        req.add_header("Content-Type", "application/json; charset=utf-8")

    try:
        with opener.open(req, timeout=timeout) as resp:
            raw = resp.read().decode("utf-8", errors="replace")
            return json.loads(raw)
    except urllib.error.HTTPError as exc:
        raw = exc.read().decode("utf-8", errors="replace")
        raise RuntimeError(f"HTTP {exc.code} for {method} {url}: {raw[:300]}") from exc


def write_captcha_png(data_uri_or_base64: str, output_path: Path) -> None:
    raw = data_uri_or_base64
    if "," in raw:
        raw = raw.split(",", 1)[1]
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_bytes(base64.b64decode(raw))


def build_cookie_header(cookie_jar: http.cookiejar.CookieJar) -> str:
    pairs: list[str] = []
    for cookie in cookie_jar:
        pairs.append(f"{cookie.name}={cookie.value}")
    return "; ".join(pairs)


def main() -> int:
    parser = argparse.ArgumentParser(description="Bootstrap PFM session cookie for diff scripts")
    parser.add_argument("--pfm-base", default="http://localhost:8000", help="PFM base URL")
    parser.add_argument("--username", default="admin", help="PFM username")
    parser.add_argument("--password", required=True, help="PFM password")
    parser.add_argument(
        "--captcha-out",
        default="scripts/validation/out/pfm_captcha.png",
        help="Path to save captcha image",
    )
    args = parser.parse_args()

    cookie_jar = http.cookiejar.CookieJar()
    opener = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(cookie_jar))

    captcha_payload = http_json(opener, args.pfm_base, "/api/utils/captcha")
    if captcha_payload.get("code") != 0:
        raise RuntimeError(f"captcha error: {captcha_payload}")

    captcha_data = captcha_payload.get("data")
    if not isinstance(captcha_data, str) and not isinstance(captcha_data, dict):
        raise RuntimeError(f"unsupported captcha payload type: {type(captcha_data).__name__}")

    if isinstance(captcha_data, dict):
        image_raw = str(captcha_data.get("image", ""))
        captcha_id = str(captcha_data.get("captchaId", ""))
    else:
        image_raw = captcha_data
        captcha_id = ""

    captcha_path = Path(args.captcha_out)
    write_captcha_png(image_raw, captcha_path)
    print(f"Captcha saved: {captcha_path}")
    print("Open the image and input captcha code.")

    code = input("Captcha code: ").strip()
    login_body: dict[str, Any] = {
        "username": args.username,
        "password": args.password,
        "code": code,
    }
    if captcha_id:
        login_body["captchaId"] = captcha_id

    login_payload = http_json(opener, args.pfm_base, "/api/utils/login", method="POST", body=login_body)
    if login_payload.get("code") != 0:
        raise RuntimeError(f"login failed: {login_payload.get('message')}")

    data = login_payload.get("data")
    token = data.get("token") if isinstance(data, dict) else None
    cookie_header = build_cookie_header(cookie_jar)

    print("\nPFM session bootstrap success.")
    if token:
        print(f"Token: {token}")
    print(f"Cookie header: {cookie_header}")
    print("\nExample commands:")
    if token:
        print(
            "python scripts/validation/ffs_pfm_homepage_diff.py "
            f"--password <PWD> --pfm-token {token} --ffs-user-id 17"
        )
    if cookie_header:
        print(
            "python scripts/validation/ffs_pfm_homepage_diff.py "
            f"--password <PWD> --pfm-cookie \"{cookie_header}\" --ffs-user-id 17"
        )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
