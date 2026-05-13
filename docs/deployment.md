# HFS Deployment Guide

## Target

- OS: AnolisOS / RHEL compatible Linux
- Topology: one host, one MySQL instance, one `hfs-backend serve`, one `hfs-backend scheduler`, one `nginx`
- Recommended minimum: `2C1G`

## Directory Layout

```text
/opt/hfs/
  backend/
  frontend/
  scripts/
/etc/hfs/
  hfs-api.env
  hfs-scheduler.env
/var/log/hfs/
```

## Build

```bash
cd /opt/hfs
bash scripts/build_release.sh
```

Artifacts:

- backend binary: `/opt/hfs/backend/target/release/hfs-backend`
- frontend static files: `/opt/hfs/frontend/dist`

## Backend Config

Use one of these methods:

1. Keep `backend/hfs.toml`
2. Or prefer environment variables through systemd `EnvironmentFile`

Recommended low-memory settings:

- `HFS__DATABASE__MIN_CONNECTIONS=0`
- `HFS__DATABASE__MAX_CONNECTIONS=4` for API
- `HFS__DATABASE__MAX_CONNECTIONS=2` for scheduler
- `HFS__DATABASE__LAZY_CONNECT=false`

## Systemd

Copy files:

```bash
cp deploy/hfs-api.service.example /etc/systemd/system/hfs-api.service
cp deploy/hfs-scheduler.service.example /etc/systemd/system/hfs-scheduler.service
cp deploy/hfs-api.env.example /etc/hfs/hfs-api.env
cp deploy/hfs-scheduler.env.example /etc/hfs/hfs-scheduler.env
mkdir -p /var/log/hfs /etc/hfs
chown -R hfs:hfs /opt/hfs /var/log/hfs
systemctl daemon-reload
systemctl enable --now hfs-api
systemctl enable --now hfs-scheduler
```

Check status:

```bash
systemctl status hfs-api
systemctl status hfs-scheduler
journalctl -u hfs-api -f
journalctl -u hfs-scheduler -f
```

## Nginx

```bash
cp deploy/nginx.hfs.conf.example /etc/nginx/conf.d/hfs.conf
nginx -t
systemctl reload nginx
```

## Database Backup

Daily cron example:

```bash
0 3 * * * /opt/hfs/scripts/backup_mysql.sh 127.0.0.1 3306 hfs hfs /data/backups replace_me >> /var/log/hfs/backup.log 2>&1
```

Restore:

```bash
bash scripts/restore_mysql.sh /data/backups/hfs-20260512-030000.sql.gz 127.0.0.1 3306 hfs hfs replace_me
```

## Release Checklist

1. Run all migrations in order.
2. Build backend and frontend in release mode.
3. Fill `/etc/hfs/*.env` with production secrets.
4. Start `hfs-api` and `hfs-scheduler`.
5. Verify `GET /api/health`.
6. Trigger one job manually from `/api/jobs/{id}/trigger` or UI.
