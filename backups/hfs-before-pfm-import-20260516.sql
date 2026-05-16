-- MySQL dump 10.13  Distrib 8.0.25, for Win64 (x86_64)
--
-- Host: localhost    Database: hfs
-- ------------------------------------------------------
-- Server version	8.0.25

/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!50503 SET NAMES utf8mb4 */;
/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;

--
-- Table structure for table `assets`
--

DROP TABLE IF EXISTS `assets`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `assets` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'Primary key',
  `user_id` bigint unsigned NOT NULL COMMENT 'Owner user id',
  `source_bill_id` bigint unsigned DEFAULT NULL COMMENT 'Source bill id',
  `name` varchar(128) NOT NULL COMMENT 'Asset name',
  `category_id` bigint unsigned NOT NULL COMMENT 'Asset category config id',
  `category_name` varchar(128) NOT NULL COMMENT 'Cached asset category name',
  `amount` decimal(18,2) NOT NULL COMMENT 'Asset amount',
  `remark` varchar(255) DEFAULT NULL COMMENT 'Remark',
  `status` varchar(32) NOT NULL DEFAULT 'active' COMMENT 'active disposed',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
  `deleted_at` datetime DEFAULT NULL COMMENT 'Soft delete time',
  PRIMARY KEY (`id`),
  KEY `idx_assets_user_created` (`user_id`,`created_at`),
  KEY `idx_assets_source_bill` (`source_bill_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='Fixed assets';
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `assets`
--

LOCK TABLES `assets` WRITE;
/*!40000 ALTER TABLE `assets` DISABLE KEYS */;
/*!40000 ALTER TABLE `assets` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `balance_calibrations`
--

DROP TABLE IF EXISTS `balance_calibrations`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `balance_calibrations` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'Primary key',
  `user_id` bigint unsigned NOT NULL COMMENT 'Owner user id',
  `calibration_date` date NOT NULL COMMENT 'Calibration date',
  `cash_balance` decimal(18,2) NOT NULL COMMENT 'Cash balance at calibration date',
  `remark` varchar(255) DEFAULT NULL COMMENT 'Remark',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
  `deleted_at` datetime DEFAULT NULL COMMENT 'Soft delete time',
  PRIMARY KEY (`id`),
  KEY `idx_balance_calibrations_user_date` (`user_id`,`calibration_date`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='Balance calibration records';
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `balance_calibrations`
--

LOCK TABLES `balance_calibrations` WRITE;
/*!40000 ALTER TABLE `balance_calibrations` DISABLE KEYS */;
/*!40000 ALTER TABLE `balance_calibrations` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `bill_tags`
--

DROP TABLE IF EXISTS `bill_tags`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `bill_tags` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'Primary key',
  `user_id` bigint unsigned NOT NULL COMMENT 'Owner user id',
  `name` varchar(64) NOT NULL COMMENT 'Tag name',
  `color` varchar(32) DEFAULT NULL COMMENT 'Tag color',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
  `deleted_at` datetime DEFAULT NULL COMMENT 'Soft delete time',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_bill_tags_user_name` (`user_id`,`name`),
  KEY `idx_bill_tags_user` (`user_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='Bill tags catalog';
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `bill_tags`
--

LOCK TABLES `bill_tags` WRITE;
/*!40000 ALTER TABLE `bill_tags` DISABLE KEYS */;
/*!40000 ALTER TABLE `bill_tags` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `bills`
--

DROP TABLE IF EXISTS `bills`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `bills` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'Primary key',
  `user_id` bigint unsigned NOT NULL COMMENT 'Owner user id',
  `account_date` date NOT NULL COMMENT 'Accounting date',
  `category_id` bigint unsigned NOT NULL COMMENT 'Config item id',
  `category_name` varchar(128) NOT NULL COMMENT 'Cached category display name',
  `bill_type` varchar(32) NOT NULL COMMENT 'income expense refund or investment action',
  `payment_method` varchar(32) NOT NULL COMMENT 'cash credit_card installment presale',
  `is_fixed_asset` tinyint(1) NOT NULL DEFAULT '0' COMMENT 'Whether this bill is for fixed asset',
  `amount` decimal(18,2) NOT NULL COMMENT 'Bill amount',
  `tags` json DEFAULT NULL COMMENT 'Bill tags json array',
  `remark` varchar(255) DEFAULT NULL COMMENT 'Remark',
  `transfer_group_id` varchar(64) DEFAULT NULL COMMENT 'Transfer pair group id',
  `transfer_target_type` varchar(32) DEFAULT NULL COMMENT 'system_user or other_person',
  `transfer_target_user_id` bigint unsigned DEFAULT NULL COMMENT 'Target user id for transfer',
  `credit_card_id` bigint unsigned DEFAULT NULL COMMENT 'Related credit card id',
  `is_installment` tinyint(1) NOT NULL DEFAULT '0' COMMENT 'Whether this bill is installment',
  `installment_months` int unsigned DEFAULT NULL COMMENT 'Installment months',
  `investment_action` varchar(32) DEFAULT NULL COMMENT 'open add reduce dividend',
  `related_investment_id` bigint unsigned DEFAULT NULL COMMENT 'Reserved investment id',
  `product_code` varchar(64) DEFAULT NULL COMMENT 'Reserved product code',
  `product_name` varchar(128) DEFAULT NULL COMMENT 'Reserved product name',
  `organization_name` varchar(128) DEFAULT NULL COMMENT 'Reserved organization name',
  `share_amount` decimal(18,6) DEFAULT NULL COMMENT 'Reserved share amount',
  `related_asset_id` bigint unsigned DEFAULT NULL COMMENT 'Reserved fixed asset id',
  `special_status` varchar(32) NOT NULL DEFAULT 'none' COMMENT 'special handling status',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
  `deleted_at` datetime DEFAULT NULL COMMENT 'Soft delete time',
  PRIMARY KEY (`id`),
  KEY `idx_bills_user_date_category` (`user_id`,`account_date`,`category_id`),
  KEY `idx_bills_transfer_group` (`transfer_group_id`),
  KEY `idx_bills_credit_card` (`credit_card_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='Accounting bills';
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `bills`
--

LOCK TABLES `bills` WRITE;
/*!40000 ALTER TABLE `bills` DISABLE KEYS */;
/*!40000 ALTER TABLE `bills` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `brands`
--

DROP TABLE IF EXISTS `brands`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `brands` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'Primary key',
  `user_id` bigint unsigned NOT NULL COMMENT 'Owner user id',
  `category_id` bigint unsigned NOT NULL COMMENT 'Brand category config id',
  `category_name` varchar(128) NOT NULL COMMENT 'Cached category name',
  `brand_name` varchar(128) NOT NULL COMMENT 'Brand name',
  `score` int NOT NULL DEFAULT '0' COMMENT 'Brand score',
  `board_type` varchar(16) NOT NULL DEFAULT 'normal' COMMENT 'red black normal',
  `review` text COMMENT 'Review content',
  `remark` varchar(255) DEFAULT NULL COMMENT 'Remark',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
  `deleted_at` datetime DEFAULT NULL COMMENT 'Soft delete time',
  PRIMARY KEY (`id`),
  KEY `idx_brands_user_category` (`user_id`,`category_id`),
  KEY `idx_brands_user_board_type` (`user_id`,`board_type`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='Brand red and black list';
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `brands`
--

LOCK TABLES `brands` WRITE;
/*!40000 ALTER TABLE `brands` DISABLE KEYS */;
/*!40000 ALTER TABLE `brands` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `budgets`
--

DROP TABLE IF EXISTS `budgets`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `budgets` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'Primary key',
  `user_id` bigint unsigned NOT NULL COMMENT 'Owner user id',
  `budget_month` date NOT NULL COMMENT 'Budget month first day',
  `category_id` bigint unsigned NOT NULL COMMENT 'Budget category config id',
  `category_name` varchar(128) NOT NULL COMMENT 'Cached budget category name',
  `planned_amount` decimal(18,2) NOT NULL DEFAULT '0.00' COMMENT 'Planned amount',
  `manual_adjusted` tinyint(1) NOT NULL DEFAULT '0' COMMENT 'Whether manually adjusted',
  `remark` varchar(255) DEFAULT NULL COMMENT 'Remark',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
  `deleted_at` datetime DEFAULT NULL COMMENT 'Soft delete time',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_budgets_user_month_category` (`user_id`,`budget_month`,`category_id`),
  KEY `idx_budgets_user_month` (`user_id`,`budget_month`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='Monthly budgets';
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `budgets`
--

LOCK TABLES `budgets` WRITE;
/*!40000 ALTER TABLE `budgets` DISABLE KEYS */;
/*!40000 ALTER TABLE `budgets` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `config_items`
--

DROP TABLE IF EXISTS `config_items`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `config_items` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'Primary key',
  `config_type` varchar(64) NOT NULL COMMENT 'Config item type',
  `name` varchar(64) NOT NULL COMMENT 'Stable item code',
  `display_name` varchar(128) NOT NULL COMMENT 'Display name',
  `is_builtin` tinyint(1) NOT NULL DEFAULT '0' COMMENT 'Whether item is built in',
  `enabled` tinyint(1) NOT NULL DEFAULT '1' COMMENT 'Whether item is enabled',
  `sort_order` int NOT NULL DEFAULT '0' COMMENT 'Sort order',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
  `deleted_at` datetime DEFAULT NULL COMMENT 'Soft delete time',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_config_items_type_name` (`config_type`,`name`),
  KEY `idx_config_items_type_sort` (`config_type`,`sort_order`)
) ENGINE=InnoDB AUTO_INCREMENT=20 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='System config items';
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `config_items`
--

LOCK TABLES `config_items` WRITE;
/*!40000 ALTER TABLE `config_items` DISABLE KEYS */;
INSERT INTO `config_items` VALUES (1,'account_category','stock','股票',1,1,1,'2026-05-16 03:10:17','2026-05-16 03:10:17',NULL),(2,'account_category','wealth','理财',1,1,2,'2026-05-16 03:10:17','2026-05-16 03:10:17',NULL),(3,'account_category','food','饮食',1,1,3,'2026-05-16 03:10:17','2026-05-16 03:10:17',NULL),(4,'account_category','living','生活',1,1,4,'2026-05-16 03:10:17','2026-05-16 03:10:17',NULL),(5,'account_category','transport','交通',1,1,5,'2026-05-16 03:10:17','2026-05-16 03:10:17',NULL),(6,'account_category','snack','零食',1,1,6,'2026-05-16 03:10:17','2026-05-16 03:10:17',NULL),(7,'account_category','housing','住房',1,1,7,'2026-05-16 03:10:17','2026-05-16 03:10:17',NULL),(8,'account_category','transfer','转账',1,1,8,'2026-05-16 03:10:17','2026-05-16 03:10:17',NULL),(9,'debt_category','credit_card','信用卡',1,1,1,'2026-05-16 03:10:17','2026-05-16 03:10:17',NULL),(10,'debt_category','loan','贷款',1,1,2,'2026-05-16 03:10:17','2026-05-16 03:10:17',NULL),(11,'debt_category','installment','分期',1,1,3,'2026-05-16 03:10:17','2026-05-16 03:10:17',NULL),(12,'presale_category','default','通用预售',1,1,1,'2026-05-16 03:10:17','2026-05-16 03:10:17',NULL),(13,'investment_category','wealth','理财',1,1,1,'2026-05-16 03:10:17','2026-05-16 03:10:17',NULL),(14,'investment_category','stock','股票',1,1,2,'2026-05-16 03:10:17','2026-05-16 03:10:17',NULL),(15,'wealth_org','default_wealth_org','默认理财机构',1,1,1,'2026-05-16 03:10:17','2026-05-16 03:10:17',NULL),(16,'stock_org','default_stock_org','默认券商',1,1,1,'2026-05-16 03:10:17','2026-05-16 03:10:17',NULL),(17,'asset_category','default_asset','通用固定资产',1,1,1,'2026-05-16 03:10:17','2026-05-16 03:10:17',NULL),(18,'budget_category','default_budget','通用预算分类',1,1,1,'2026-05-16 03:10:17','2026-05-16 03:10:17',NULL),(19,'brand_category','default_brand','通用品牌分类',1,1,1,'2026-05-16 03:10:17','2026-05-16 03:10:17',NULL);
/*!40000 ALTER TABLE `config_items` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `credit_cards`
--

DROP TABLE IF EXISTS `credit_cards`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `credit_cards` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'Primary key',
  `user_id` bigint unsigned NOT NULL COMMENT 'Owner user id',
  `name` varchar(64) NOT NULL COMMENT 'Credit card name',
  `billing_day` tinyint unsigned NOT NULL COMMENT 'Billing day in month',
  `repayment_day` tinyint unsigned NOT NULL COMMENT 'Repayment day in month',
  `credit_limit` decimal(18,2) NOT NULL DEFAULT '0.00' COMMENT 'Credit limit',
  `enabled` tinyint(1) NOT NULL DEFAULT '1' COMMENT 'Whether card is enabled',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
  `deleted_at` datetime DEFAULT NULL COMMENT 'Soft delete time',
  PRIMARY KEY (`id`),
  KEY `idx_credit_cards_user_enabled` (`user_id`,`enabled`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='Credit card configurations';
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `credit_cards`
--

LOCK TABLES `credit_cards` WRITE;
/*!40000 ALTER TABLE `credit_cards` DISABLE KEYS */;
/*!40000 ALTER TABLE `credit_cards` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `dashboard_snapshots`
--

DROP TABLE IF EXISTS `dashboard_snapshots`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `dashboard_snapshots` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'Primary key',
  `user_id` bigint unsigned NOT NULL COMMENT 'Owner user id',
  `snapshot_date` date NOT NULL COMMENT 'Snapshot date',
  `cash_balance` decimal(18,2) NOT NULL DEFAULT '0.00' COMMENT 'Cash balance',
  `total_assets` decimal(18,2) NOT NULL DEFAULT '0.00' COMMENT 'Total assets',
  `outstanding_amount` decimal(18,2) NOT NULL DEFAULT '0.00' COMMENT 'Outstanding amount',
  `wealth_amount` decimal(18,2) NOT NULL DEFAULT '0.00' COMMENT 'Wealth amount',
  `stock_amount` decimal(18,2) NOT NULL DEFAULT '0.00' COMMENT 'Stock amount',
  `income` decimal(18,2) NOT NULL DEFAULT '0.00' COMMENT 'Income in range',
  `expense` decimal(18,2) NOT NULL DEFAULT '0.00' COMMENT 'Expense in range',
  `net_cash_flow` decimal(18,2) NOT NULL DEFAULT '0.00' COMMENT 'Net cash flow in range',
  `calibration_status` varchar(32) NOT NULL DEFAULT 'uncalibrated' COMMENT 'Calibration status',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
  `deleted_at` datetime DEFAULT NULL COMMENT 'Soft delete time',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_dashboard_snapshots_user_date` (`user_id`,`snapshot_date`),
  KEY `idx_dashboard_snapshots_snapshot_date` (`snapshot_date`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='Dashboard daily snapshots';
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `dashboard_snapshots`
--

LOCK TABLES `dashboard_snapshots` WRITE;
/*!40000 ALTER TABLE `dashboard_snapshots` DISABLE KEYS */;
/*!40000 ALTER TABLE `dashboard_snapshots` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `debts`
--

DROP TABLE IF EXISTS `debts`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `debts` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'Primary key',
  `user_id` bigint unsigned NOT NULL COMMENT 'Owner user id',
  `source_bill_id` bigint unsigned DEFAULT NULL COMMENT 'Source bill id',
  `start_date` date NOT NULL COMMENT 'Debt start date',
  `end_date` date DEFAULT NULL COMMENT 'Debt end date',
  `repay_deadline` date DEFAULT NULL COMMENT 'Repayment deadline',
  `category_id` bigint unsigned NOT NULL COMMENT 'Debt category config id',
  `category_name` varchar(128) NOT NULL COMMENT 'Cached debt category name',
  `amount` decimal(18,2) NOT NULL COMMENT 'Debt amount',
  `period_count` int unsigned NOT NULL DEFAULT '1' COMMENT 'Number of periods',
  `period_unit` varchar(16) NOT NULL DEFAULT 'month' COMMENT 'day month year',
  `period_value` int unsigned NOT NULL DEFAULT '1' COMMENT 'Period value',
  `payment_method` varchar(32) NOT NULL COMMENT 'cash or credit_card',
  `status` varchar(32) NOT NULL DEFAULT 'pending' COMMENT 'pending settled cancelled',
  `remark` varchar(255) DEFAULT NULL COMMENT 'Remark',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
  `deleted_at` datetime DEFAULT NULL COMMENT 'Soft delete time',
  PRIMARY KEY (`id`),
  KEY `idx_debts_user_start` (`user_id`,`start_date`),
  KEY `idx_debts_source_bill` (`source_bill_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='Debt records';
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `debts`
--

LOCK TABLES `debts` WRITE;
/*!40000 ALTER TABLE `debts` DISABLE KEYS */;
/*!40000 ALTER TABLE `debts` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `intel_items`
--

DROP TABLE IF EXISTS `intel_items`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `intel_items` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'Primary key',
  `user_id` bigint unsigned NOT NULL COMMENT 'Owner user id',
  `title` varchar(128) NOT NULL COMMENT 'Intel title',
  `source` varchar(128) DEFAULT NULL COMMENT 'Intel source',
  `item_date` date NOT NULL COMMENT 'Intel date',
  `status` varchar(32) NOT NULL DEFAULT 'draft' COMMENT 'draft active archived',
  `tags` varchar(255) DEFAULT NULL COMMENT 'Comma separated tags',
  `summary` varchar(255) DEFAULT NULL COMMENT 'Short summary',
  `content` text COMMENT 'Intel content',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
  `deleted_at` datetime DEFAULT NULL COMMENT 'Soft delete time',
  PRIMARY KEY (`id`),
  KEY `idx_intel_items_user_date` (`user_id`,`item_date`),
  KEY `idx_intel_items_status` (`status`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='Intel placeholder items';
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `intel_items`
--

LOCK TABLES `intel_items` WRITE;
/*!40000 ALTER TABLE `intel_items` DISABLE KEYS */;
/*!40000 ALTER TABLE `intel_items` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `investment_transactions`
--

DROP TABLE IF EXISTS `investment_transactions`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `investment_transactions` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'Primary key',
  `investment_id` bigint unsigned NOT NULL COMMENT 'Investment id',
  `source_bill_id` bigint unsigned NOT NULL COMMENT 'Source bill id',
  `transaction_date` date NOT NULL COMMENT 'Transaction date',
  `action` varchar(32) NOT NULL COMMENT 'open_position add_position reduce_position dividend',
  `shares` decimal(18,6) NOT NULL DEFAULT '0.000000' COMMENT 'Transaction share amount',
  `amount` decimal(18,2) NOT NULL DEFAULT '0.00' COMMENT 'Transaction amount',
  `unit_price` decimal(18,6) NOT NULL DEFAULT '0.000000' COMMENT 'Unit price from transaction',
  `realized_profit` decimal(18,2) NOT NULL DEFAULT '0.00' COMMENT 'Realized profit for this transaction',
  `remark` varchar(255) DEFAULT NULL COMMENT 'Remark',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
  `deleted_at` datetime DEFAULT NULL COMMENT 'Soft delete time',
  PRIMARY KEY (`id`),
  KEY `idx_investment_transactions_investment_date` (`investment_id`,`transaction_date`),
  KEY `idx_investment_transactions_source_bill` (`source_bill_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='Investment transactions';
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `investment_transactions`
--

LOCK TABLES `investment_transactions` WRITE;
/*!40000 ALTER TABLE `investment_transactions` DISABLE KEYS */;
/*!40000 ALTER TABLE `investment_transactions` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `investments`
--

DROP TABLE IF EXISTS `investments`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `investments` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'Primary key',
  `user_id` bigint unsigned NOT NULL COMMENT 'Owner user id',
  `source_bill_id` bigint unsigned DEFAULT NULL COMMENT 'First source bill id',
  `investment_type` varchar(16) NOT NULL COMMENT 'stock or wealth',
  `name` varchar(128) NOT NULL COMMENT 'Investment product name',
  `code` varchar(64) NOT NULL COMMENT 'Investment product code',
  `organization_name` varchar(128) NOT NULL COMMENT 'Organization or broker name',
  `market` varchar(64) DEFAULT NULL COMMENT 'Reserved market field',
  `total_shares` decimal(18,6) NOT NULL DEFAULT '0.000000' COMMENT 'Current total shares',
  `total_cost` decimal(18,2) NOT NULL DEFAULT '0.00' COMMENT 'Current total cost',
  `average_cost` decimal(18,6) NOT NULL DEFAULT '0.000000' COMMENT 'Average cost per share',
  `current_price` decimal(18,6) NOT NULL DEFAULT '0.000000' COMMENT 'Latest price or nav',
  `market_value` decimal(18,2) NOT NULL DEFAULT '0.00' COMMENT 'Current market value',
  `realized_profit` decimal(18,2) NOT NULL DEFAULT '0.00' COMMENT 'Realized profit amount',
  `unrealized_profit` decimal(18,2) NOT NULL DEFAULT '0.00' COMMENT 'Unrealized profit amount',
  `total_profit` decimal(18,2) NOT NULL DEFAULT '0.00' COMMENT 'Total profit amount',
  `total_profit_rate` decimal(18,6) NOT NULL DEFAULT '0.000000' COMMENT 'Total profit rate',
  `status` varchar(32) NOT NULL DEFAULT 'holding' COMMENT 'holding sold archived',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
  `deleted_at` datetime DEFAULT NULL COMMENT 'Soft delete time',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_investments_user_type_code` (`user_id`,`investment_type`,`code`),
  KEY `idx_investments_user_status` (`user_id`,`status`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='Investments';
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `investments`
--

LOCK TABLES `investments` WRITE;
/*!40000 ALTER TABLE `investments` DISABLE KEYS */;
/*!40000 ALTER TABLE `investments` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `job_configs`
--

DROP TABLE IF EXISTS `job_configs`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `job_configs` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'Primary key',
  `job_code` varchar(64) NOT NULL COMMENT 'Unique job code',
  `job_name` varchar(128) NOT NULL COMMENT 'Display name',
  `cron_expr` varchar(64) NOT NULL COMMENT 'Cron expression',
  `enabled` tinyint(1) NOT NULL DEFAULT '1' COMMENT 'Whether the job is enabled',
  `batch_size` int unsigned NOT NULL DEFAULT '100' COMMENT 'Batch size for each run',
  `concurrency` int unsigned NOT NULL DEFAULT '1' COMMENT 'Max concurrent workers',
  `timeout_seconds` int unsigned NOT NULL DEFAULT '300' COMMENT 'Timeout for one run',
  `retry_count` int unsigned NOT NULL DEFAULT '0' COMMENT 'Retry count on failure',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
  `deleted_at` datetime DEFAULT NULL COMMENT 'Soft delete time',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_job_configs_job_code` (`job_code`)
) ENGINE=InnoDB AUTO_INCREMENT=5 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='Scheduler job configuration';
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `job_configs`
--

LOCK TABLES `job_configs` WRITE;
/*!40000 ALTER TABLE `job_configs` DISABLE KEYS */;
INSERT INTO `job_configs` VALUES (1,'stock_market_sync_daily','全市场股票同步','0 0 1 * * * *',1,100,1,300,0,'2026-05-16 03:10:17','2026-05-16 03:10:17',NULL),(2,'wealth_sync_daily_slots','理财产品同步','0 0 */6 * * * *',1,100,1,300,0,'2026-05-16 03:10:17','2026-05-16 03:10:17',NULL),(3,'stock_realtime_sync','交易时段股票同步','0 */5 * * * * *',1,100,1,300,0,'2026-05-16 03:10:17','2026-05-16 03:10:17',NULL),(4,'dashboard_snapshot_daily','首页快照','0 10 23 * * * *',1,100,1,300,0,'2026-05-16 03:10:17','2026-05-16 03:10:17',NULL);
/*!40000 ALTER TABLE `job_configs` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `job_runs`
--

DROP TABLE IF EXISTS `job_runs`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `job_runs` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'Primary key',
  `job_id` bigint unsigned NOT NULL COMMENT 'Job config id',
  `status` varchar(32) NOT NULL COMMENT 'Run status',
  `scheduled_at` datetime DEFAULT NULL COMMENT 'Scheduled execution time',
  `trigger_type` varchar(32) NOT NULL DEFAULT 'scheduler' COMMENT 'scheduler manual bootstrap',
  `started_at` datetime NOT NULL COMMENT 'Run start time',
  `finished_at` datetime DEFAULT NULL COMMENT 'Run end time',
  `duration_ms` bigint unsigned DEFAULT NULL COMMENT 'Duration in milliseconds',
  `message` varchar(255) DEFAULT NULL COMMENT 'Summary message',
  `error_message` text COMMENT 'Failure details',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
  `deleted_at` datetime DEFAULT NULL COMMENT 'Soft delete time',
  PRIMARY KEY (`id`),
  KEY `idx_job_runs_job_started` (`job_id`,`started_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='Scheduler job run logs';
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `job_runs`
--

LOCK TABLES `job_runs` WRITE;
/*!40000 ALTER TABLE `job_runs` DISABLE KEYS */;
/*!40000 ALTER TABLE `job_runs` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `presales`
--

DROP TABLE IF EXISTS `presales`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `presales` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'Primary key',
  `user_id` bigint unsigned NOT NULL COMMENT 'Owner user id',
  `source_bill_id` bigint unsigned DEFAULT NULL COMMENT 'Source bill id',
  `deposit_date` date NOT NULL COMMENT 'Deposit date',
  `final_payment_date` date DEFAULT NULL COMMENT 'Final payment date',
  `category_id` bigint unsigned NOT NULL COMMENT 'Presale category config id',
  `category_name` varchar(128) NOT NULL COMMENT 'Cached presale category name',
  `deposit_amount` decimal(18,2) NOT NULL COMMENT 'Deposit amount',
  `final_payment_amount` decimal(18,2) NOT NULL DEFAULT '0.00' COMMENT 'Final payment amount',
  `status` varchar(32) NOT NULL DEFAULT 'pending_final_payment' COMMENT 'pending_final_payment paid_final_payment cancelled',
  `remark` varchar(255) DEFAULT NULL COMMENT 'Remark',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
  `deleted_at` datetime DEFAULT NULL COMMENT 'Soft delete time',
  PRIMARY KEY (`id`),
  KEY `idx_presales_user_deposit` (`user_id`,`deposit_date`),
  KEY `idx_presales_source_bill` (`source_bill_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='Presale records';
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `presales`
--

LOCK TABLES `presales` WRITE;
/*!40000 ALTER TABLE `presales` DISABLE KEYS */;
/*!40000 ALTER TABLE `presales` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `stock_indicators`
--

DROP TABLE IF EXISTS `stock_indicators`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `stock_indicators` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'Primary key',
  `investment_id` bigint unsigned NOT NULL COMMENT 'Investment id',
  `user_id` bigint unsigned NOT NULL COMMENT 'Owner user id',
  `code` varchar(64) NOT NULL COMMENT 'Investment code',
  `indicator_date` date NOT NULL COMMENT 'Indicator date',
  `current_price` decimal(18,6) NOT NULL DEFAULT '0.000000' COMMENT 'Latest price',
  `price_change_rate` decimal(18,6) NOT NULL DEFAULT '0.000000' COMMENT 'Price change rate',
  `profit_rate` decimal(18,6) NOT NULL DEFAULT '0.000000' COMMENT 'Holding profit rate',
  `ma_bias` decimal(18,6) NOT NULL DEFAULT '0.000000' COMMENT 'Moving average bias proxy',
  `volume_ratio` decimal(18,6) NOT NULL DEFAULT '1.000000' COMMENT 'Volume ratio proxy',
  `score` int NOT NULL DEFAULT '0' COMMENT 'Computed score',
  `suggestion` varchar(32) NOT NULL DEFAULT '???' COMMENT 'Buy watch reduce sell cooldown',
  `reason` varchar(255) NOT NULL COMMENT 'Reason summary',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
  `deleted_at` datetime DEFAULT NULL COMMENT 'Soft delete time',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_stock_indicators_investment_date` (`investment_id`,`indicator_date`),
  KEY `idx_stock_indicators_user_date_score` (`user_id`,`indicator_date`,`score`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='Stock indicators and scores';
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `stock_indicators`
--

LOCK TABLES `stock_indicators` WRITE;
/*!40000 ALTER TABLE `stock_indicators` DISABLE KEYS */;
/*!40000 ALTER TABLE `stock_indicators` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `strategy_configs`
--

DROP TABLE IF EXISTS `strategy_configs`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `strategy_configs` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'Primary key',
  `user_id` bigint unsigned NOT NULL COMMENT 'Owner user id',
  `investment_type` varchar(16) NOT NULL COMMENT 'stock or wealth',
  `target_code` varchar(64) DEFAULT NULL COMMENT 'Specific investment code, null means generic strategy',
  `strategy_name` varchar(128) NOT NULL COMMENT 'Strategy display name',
  `enabled` tinyint(1) NOT NULL DEFAULT '1' COMMENT 'Whether strategy is enabled',
  `risk_level` varchar(16) NOT NULL DEFAULT 'balanced' COMMENT 'low balanced high',
  `preferred_min_score` int NOT NULL DEFAULT '60' COMMENT 'Minimum score for positive suggestion',
  `cooldown_days` int NOT NULL DEFAULT '3' COMMENT 'Cooling days after opening position',
  `take_profit_rate` decimal(18,6) NOT NULL DEFAULT '0.150000' COMMENT 'Take profit rate threshold',
  `stop_loss_rate` decimal(18,6) NOT NULL DEFAULT '0.080000' COMMENT 'Stop loss rate threshold',
  `notes` varchar(255) DEFAULT NULL COMMENT 'Notes',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
  `deleted_at` datetime DEFAULT NULL COMMENT 'Soft delete time',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_strategy_configs_user_type_code` (`user_id`,`investment_type`,`target_code`),
  KEY `idx_strategy_configs_user_type_enabled` (`user_id`,`investment_type`,`enabled`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='Strategy configs';
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `strategy_configs`
--

LOCK TABLES `strategy_configs` WRITE;
/*!40000 ALTER TABLE `strategy_configs` DISABLE KEYS */;
/*!40000 ALTER TABLE `strategy_configs` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `users`
--

DROP TABLE IF EXISTS `users`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `users` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'Primary key',
  `username` varchar(64) NOT NULL COMMENT 'Login username',
  `display_name` varchar(64) NOT NULL COMMENT 'Display name',
  `password_hash` varchar(255) NOT NULL COMMENT 'Password hash',
  `role` varchar(32) NOT NULL DEFAULT 'member' COMMENT 'User role',
  `enabled` tinyint(1) NOT NULL DEFAULT '1' COMMENT 'Whether the user is enabled',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
  `deleted_at` datetime DEFAULT NULL COMMENT 'Soft delete time',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_users_username` (`username`)
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='System users';
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `users`
--

LOCK TABLES `users` WRITE;
/*!40000 ALTER TABLE `users` DISABLE KEYS */;
INSERT INTO `users` VALUES (1,'admin','Administrator','sha256$8c6976e5b5410415bde908bd4dee15dfb167a9c873fc4bb8a81f6f2ab448a918','owner',1,'2026-05-16 03:10:17','2026-05-16 03:10:17',NULL);
/*!40000 ALTER TABLE `users` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `wealth_indicators`
--

DROP TABLE IF EXISTS `wealth_indicators`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `wealth_indicators` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'Primary key',
  `investment_id` bigint unsigned NOT NULL COMMENT 'Investment id',
  `user_id` bigint unsigned NOT NULL COMMENT 'Owner user id',
  `code` varchar(64) NOT NULL COMMENT 'Investment code',
  `indicator_date` date NOT NULL COMMENT 'Indicator date',
  `current_nav` decimal(18,6) NOT NULL DEFAULT '0.000000' COMMENT 'Latest nav',
  `annualized_return_1d` decimal(18,6) NOT NULL DEFAULT '0.000000' COMMENT 'Annualized proxy 1d',
  `annualized_return_7d` decimal(18,6) NOT NULL DEFAULT '0.000000' COMMENT 'Annualized proxy 7d',
  `annualized_return_30d` decimal(18,6) NOT NULL DEFAULT '0.000000' COMMENT 'Annualized proxy 30d',
  `drawdown_proxy` decimal(18,6) NOT NULL DEFAULT '0.000000' COMMENT 'Drawdown proxy',
  `profit_rate` decimal(18,6) NOT NULL DEFAULT '0.000000' COMMENT 'Holding profit rate',
  `score` int NOT NULL DEFAULT '0' COMMENT 'Computed score',
  `suggestion` varchar(32) NOT NULL DEFAULT '???' COMMENT 'DCA hold take_profit watch',
  `reason` varchar(255) NOT NULL COMMENT 'Reason summary',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
  `deleted_at` datetime DEFAULT NULL COMMENT 'Soft delete time',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_wealth_indicators_investment_date` (`investment_id`,`indicator_date`),
  KEY `idx_wealth_indicators_user_date_score` (`user_id`,`indicator_date`,`score`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='Wealth indicators and scores';
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `wealth_indicators`
--

LOCK TABLES `wealth_indicators` WRITE;
/*!40000 ALTER TABLE `wealth_indicators` DISABLE KEYS */;
/*!40000 ALTER TABLE `wealth_indicators` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

-- Dump completed on 2026-05-16 11:35:38
