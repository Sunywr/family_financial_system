# FFS项目 - 债务(Debt)模块完整分析

## 1. 前端代码结构

### 主视图
- **文件**: [frontend/src/views/DebtsView.vue](frontend/src/views/DebtsView.vue)
- **功能**: 债务主页面，展示债务列表
- **主要特性**:
  - 全文检索（分类/备注/ID）
  - 分页显示（支持20/50/100条每页）
  - 状态显示：待还(pending) / 已还(settled) / 已取消(cancelled)
  - 周期单位转换：day/week/month/year

### API接口
- **文件**: [frontend/src/api/debts.ts](frontend/src/api/debts.ts)
- **接口函数**: `fetchDebts(keyword, page, pageSize)`
- **API端点**: `GET /debts`
- **请求参数**:
  ```typescript
  {
    page: number
    page_size: number
    keyword: string (optional)
    user_id: number (当前用户ID)
  }
  ```

### 数据模型
```typescript
interface Debt {
  id: number
  user_id: number
  source_bill_id?: number | null
  start_date: string
  end_date?: string | null
  repay_deadline?: string | null
  category_name: string
  amount: string
  period_count: number
  period_unit: string
  payment_method: string
  status: string
  remark?: string | null
}
```

## 2. 后端代码结构 (Rust)

### 处理器 (Handler)
- **文件**: [backend/src/handler/debts.rs](backend/src/handler/debts.rs)
- **端点列表**:
  - `GET /debts` - 列出所有债务
  - `GET /debts/:id` - 获取债务详情
  - `POST /debts` - 创建新债务
  - `PUT /debts/:id` - 更新债务
  - `DELETE /debts/:id` - 删除债务

### 业务逻辑服务
- **文件**: [backend/src/service/debt_service.rs](backend/src/service/debt_service.rs)
- **主要函数**:
  - `list()` - 获取债务列表（支持权限控制）
  - `detail()` - 获取债务详情
  - `create()` - 创建债务
  - `update()` - 更新债务
  - `delete()` - 删除债务
- **验证逻辑**:
  - 分类必须为 `debt_category` 类型
  - 金额必须为正数
  - 周期参数验证
  - 周期单位必须为 day/month/year

### 数据传输对象 (DTO)
- **文件**: [backend/src/dto/debt.rs](backend/src/dto/debt.rs)

**查询参数**:
```rust
pub struct DebtListQuery {
    pub pagination: PaginationQuery,
    pub user_id: Option<u64>,
    pub status: Option<String>,
    pub keyword: Option<String>,
}
```

**创建请求**:
```rust
pub struct CreateDebtRequest {
    pub user_id: u64,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub repay_deadline: Option<NaiveDate>,
    pub category_id: u64,
    pub amount: String,
    pub period_count: u32,
    pub period_unit: String,
    pub period_value: u32,
    pub payment_method: String,
    pub remark: Option<String>,
}
```

**更新请求**:
```rust
pub struct UpdateDebtRequest {
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub repay_deadline: Option<NaiveDate>,
    pub category_id: u64,
    pub amount: String,
    pub period_count: u32,
    pub period_unit: String,
    pub period_value: u32,
    pub payment_method: String,
    pub status: String,
    pub remark: Option<String>,
}
```

### 数据模型
- **文件**: [backend/src/model/debt.rs](backend/src/model/debt.rs)

```rust
pub struct Debt {
    pub id: u64,
    pub user_id: u64,
    pub source_bill_id: Option<u64>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub repay_deadline: Option<NaiveDate>,
    pub category_id: u64,
    pub category_name: String,
    pub amount: String,
    pub period_count: u32,
    pub period_unit: String,
    pub period_value: u32,
    pub payment_method: String,
    pub status: String,
    pub remark: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
```

## 3. 数据库表结构

### Debts表
- **文件**: [migrations/0004_phase4_debts_presales_assets.sql](migrations/0004_phase4_debts_presales_assets.sql)

```sql
CREATE TABLE debts (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT COMMENT 'Primary key',
    user_id BIGINT UNSIGNED NOT NULL COMMENT 'Owner user id',
    source_bill_id BIGINT UNSIGNED NULL COMMENT 'Source bill id',
    start_date DATE NOT NULL COMMENT 'Debt start date',
    end_date DATE NULL COMMENT 'Debt end date',
    repay_deadline DATE NULL COMMENT 'Repayment deadline',
    category_id BIGINT UNSIGNED NOT NULL COMMENT 'Debt category config id',
    category_name VARCHAR(128) NOT NULL COMMENT 'Cached debt category name',
    amount DECIMAL(18,2) NOT NULL COMMENT 'Debt amount',
    period_count INT UNSIGNED NOT NULL DEFAULT 1 COMMENT 'Number of periods',
    period_unit VARCHAR(16) NOT NULL DEFAULT 'month' COMMENT 'day month year',
    period_value INT UNSIGNED NOT NULL DEFAULT 1 COMMENT 'Period value',
    payment_method VARCHAR(32) NOT NULL COMMENT 'cash or credit_card',
    status VARCHAR(32) NOT NULL DEFAULT 'pending' COMMENT 'pending settled cancelled',
    remark VARCHAR(255) NULL COMMENT 'Remark',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
    deleted_at DATETIME NULL COMMENT 'Soft delete time',
    KEY idx_debts_user_start (user_id, start_date),
    KEY idx_debts_source_bill (source_bill_id)
) COMMENT='Debt records';
```

### 关键字段说明
| 字段 | 类型 | 说明 | 备注 |
|------|------|------|------|
| id | BIGINT | 主键 | 自增 |
| user_id | BIGINT | 所有者用户ID | 非空 |
| source_bill_id | BIGINT | 来源账单ID | 可空，用于关联账单 |
| start_date | DATE | 债务开始日期 | 非空 |
| end_date | DATE | 债务结束日期 | 可空 |
| repay_deadline | DATE | 还款截止日期 | 可空 |
| category_id | BIGINT | 分类ID | 非空，外键 |
| category_name | VARCHAR(128) | 分类名称缓存 | 非空 |
| amount | DECIMAL(18,2) | 债务金额 | 非空 |
| period_count | INT | 周期数 | 默认1 |
| period_unit | VARCHAR(16) | 周期单位 | day/month/year，默认month |
| period_value | INT | 周期值 | 默认1 |
| payment_method | VARCHAR(32) | 支付方式 | cash 或 credit_card |
| status | VARCHAR(32) | 状态 | pending/settled/cancelled |
| remark | VARCHAR(255) | 备注 | 可空 |
| created_at | DATETIME | 创建时间 | 自动时间戳 |
| updated_at | DATETIME | 更新时间 | 自动更新时间戳 |
| deleted_at | DATETIME | 删除时间 | 软删除标记 |

### 重要发现：关于信用卡标识
**❌ FFS项目的debts表中没有专门的信用卡标识字段**
- `payment_method` 字段记录支付方式（cash或credit_card），但不关联具体的信用卡记录
- 与PFM项目的实现方式不同

## 4. 与账单(Bills)的关联

- **相关迁移**: [migrations/0012_phase13_bill_related_debt.sql](migrations/0012_phase13_bill_related_debt.sql)
- Bills表中添加了 `related_debt_id` 字段
- 用于关联账单到特定债务
- 在仪表板中用于计算周期债务统计

## 5. 当前债务显示方式

### 前端显示
- **列表视图**: 以表格形式展示
- **显示字段**: ID、开始日期、结束日期、还款截止日期、分类、金额、单位、状态、备注
- **状态颜色编码**: pending(黄色) / settled(绿色)
- **周期单位翻译**: day→天、week→周、month→月、year→年

### 仪表板集成
- 债务可从仪表板钻取查看
- 支持通过以下维度过滤：
  - 周期债务摘要
  - 周期债务明细
  - 按日期范围计算债务余额

## 6. 权限模型

- **Admin用户**: 可查看所有用户的债务
- **普通用户**: 只能查看自己的债务
- **所有权限检查**: 在service层进行，确保数据安全

## 7. 相关文件关系图

```
Frontend
  ├── DebtsView.vue (视图展示)
  ├── debts.ts (API调用)
  └── Dashboard (仪表板集成)

Backend
  ├── handler/debts.rs (HTTP处理)
  ├── service/debt_service.rs (业务逻辑)
  ├── dto/debt.rs (数据传输)
  ├── model/debt.rs (数据模型)
  └── repository/debt_repository.rs (数据访问)

Database
  ├── debts (主表)
  ├── bills (通过related_debt_id关联)
  ├── config_items (分类配置)
  └── users (所有者关联)
```

## 8. API调用示例

### 获取债务列表
```bash
GET /debts?page=1&page_size=20&keyword=&user_id=1
Authorization: Bearer <token>
```

**响应示例**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "page": 1,
    "page_size": 20,
    "total": 5,
    "list": [
      {
        "id": 1,
        "user_id": 1,
        "source_bill_id": null,
        "start_date": "2026-05-01",
        "end_date": "2026-05-31",
        "repay_deadline": "2026-06-10",
        "category_name": "信用卡账单",
        "amount": "5000.00",
        "period_count": 1,
        "period_unit": "month",
        "payment_method": "credit_card",
        "status": "pending",
        "remark": "五月账单"
      }
    ]
  }
}
```

### 创建债务
```bash
POST /debts
Content-Type: application/json
Authorization: Bearer <token>

{
  "user_id": 1,
  "start_date": "2026-05-01",
  "end_date": "2026-05-31",
  "repay_deadline": "2026-06-10",
  "category_id": 1,
  "amount": "5000.00",
  "period_count": 1,
  "period_unit": "month",
  "period_value": 1,
  "payment_method": "credit_card",
  "remark": "五月账单"
}
```

## 9. 注意事项

1. **金额精度**: 使用 Decimal(18,2) 以避免浮点精度问题
2. **软删除**: 债务使用软删除，deleted_at 为空表示未删除
3. **缓存字段**: category_name 是缓存字段，改进查询性能
4. **日期范围**: start_date 和 end_date 用于定义债务周期
5. **周期计算**: period_count + period_unit + period_value 组合定义还款周期
6. **无信用卡详情关联**: FFS的debt表不包含具体的信用卡对象关联，只有支付方式标记

