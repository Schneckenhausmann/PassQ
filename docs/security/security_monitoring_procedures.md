# PassQ Security Monitoring Procedures

## Executive Summary

This document establishes comprehensive security monitoring procedures for PassQ, ensuring continuous visibility into security events, threats, and system health. These procedures enable proactive threat detection, rapid incident response, and compliance with security standards.

## Monitoring Architecture

### Security Monitoring Stack

```
┌─────────────────────────────────────────────────────────────┐
│                    Security Operations Center (SOC)         │
├─────────────────────────────────────────────────────────────┤
│  Security Analysts  │  Incident Response  │  Threat Hunters │
└─────────────────────┬───────────────────────────────────────┘
                      │
┌─────────────────────┴───────────────────────────────────────┐
│                SIEM Platform (Splunk/ELK)                   │
├─────────────────────────────────────────────────────────────┤
│  • Event Correlation    • Threat Intelligence              │
│  • Alert Management     • Compliance Reporting             │
│  • Dashboards          • Forensic Analysis                 │
└─────────────────────┬───────────────────────────────────────┘
                      │
┌─────────────────────┴───────────────────────────────────────┐
│                   Log Aggregation Layer                     │
├─────────────────────────────────────────────────────────────┤
│  Logstash/Fluentd  │  Kafka  │  Redis  │  Elasticsearch    │
└─────────────────────┬───────────────────────────────────────┘
                      │
┌─────────────────────┴───────────────────────────────────────┐
│                    Data Sources                             │
├─────────────────────┬───────────────┬───────────────────────┤
│   Application Logs  │ System Logs   │  Network Logs         │
│   • API Logs        │ • OS Events   │  • Firewall Logs     │
│   • Auth Logs       │ • Process     │  • IDS/IPS Alerts    │
│   • Error Logs      │ • File Access │  • DNS Logs          │
│   • Audit Logs      │ • Registry    │  • Proxy Logs        │
└─────────────────────┴───────────────┴───────────────────────┘
```

### Monitoring Components

#### 1. Security Information and Event Management (SIEM)
- **Platform**: Splunk Enterprise Security / ELK Stack
- **Purpose**: Centralized log analysis and correlation
- **Retention**: 90 days hot, 1 year warm, 7 years cold
- **Capacity**: 10GB/day average, 50GB/day peak

#### 2. Intrusion Detection System (IDS)
- **Network IDS**: Suricata with custom rules
- **Host IDS**: OSSEC agents on all systems
- **Cloud IDS**: AWS GuardDuty / Azure Sentinel
- **Coverage**: 100% of network traffic and critical hosts

#### 3. Vulnerability Management
- **Scanner**: Nessus Professional
- **Frequency**: Weekly automated scans
- **Scope**: All internet-facing and internal systems
- **Integration**: SIEM correlation with vulnerability data

#### 4. Threat Intelligence
- **Feeds**: Commercial and open-source threat feeds
- **Integration**: Automated IOC matching in SIEM
- **Sources**: FBI, DHS, industry-specific feeds
- **Update Frequency**: Real-time for critical threats

## Log Sources and Collection

### Application Logs

#### PassQ Backend (Rust/Actix)
```rust
// Structured logging configuration
use slog::{Drain, Logger};
use slog_json::Json;

// Security event logging
info!(logger, "Authentication attempt"; 
    "user_id" => user_id,
    "ip_address" => client_ip,
    "user_agent" => user_agent,
    "result" => "success",
    "timestamp" => Utc::now().to_rfc3339()
);

warn!(logger, "Suspicious activity detected";
    "event_type" => "multiple_failed_logins",
    "user_id" => user_id,
    "attempt_count" => attempt_count,
    "time_window" => "5_minutes"
);
```

#### Critical Events to Log
1. **Authentication Events**
   - Login attempts (success/failure)
   - Password changes
   - MFA events
   - Session creation/termination
   - Account lockouts

2. **Authorization Events**
   - Permission changes
   - Role modifications
   - Access denials
   - Privilege escalations

3. **Data Access Events**
   - Password vault access
   - Data exports
   - Bulk operations
   - API calls
   - Database queries

4. **System Events**
   - Application starts/stops
   - Configuration changes
   - Error conditions
   - Performance anomalies

### System Logs

#### Linux System Monitoring
```bash
# Rsyslog configuration for security events
# /etc/rsyslog.d/50-security.conf

# Authentication logs
auth,authpriv.*                 @@siem.passq.com:514

# System logs
kern.*                          @@siem.passq.com:514

# Audit logs
$ModLoad imfile
$InputFileName /var/log/audit/audit.log
$InputFileTag audit:
$InputFileStateFile audit
$InputFileSeverity info
$InputFileFacility local6
$InputRunFileMonitor
```

#### Windows System Monitoring
```powershell
# PowerShell script for Windows Event Log forwarding
# Configure Windows Event Collector

wecutil cs security-subscription.xml

# Security events to monitor
# 4624 - Successful logon
# 4625 - Failed logon
# 4648 - Logon using explicit credentials
# 4672 - Special privileges assigned
# 4720 - User account created
```

### Network Logs

#### Firewall Configuration
```bash
# iptables logging configuration
iptables -A INPUT -j LOG --log-prefix "FIREWALL-INPUT: "
iptables -A OUTPUT -j LOG --log-prefix "FIREWALL-OUTPUT: "
iptables -A FORWARD -j LOG --log-prefix "FIREWALL-FORWARD: "

# Send logs to SIEM
logger -p local0.info "Firewall rule triggered: $LOG_MESSAGE"
```

#### Network Traffic Analysis
```yaml
# Suricata configuration
suricata:
  outputs:
    - eve-log:
        enabled: yes
        filetype: syslog
        facility: local5
        level: info
        types:
          - alert
          - http
          - dns
          - tls
          - files
          - smtp
```

## Monitoring Rules and Alerts

### Authentication Monitoring

#### Failed Login Attempts
```sql
-- SIEM Rule: Multiple Failed Logins
SELECT user_id, ip_address, COUNT(*) as failed_attempts
FROM authentication_logs 
WHERE result = 'failure' 
  AND timestamp > NOW() - INTERVAL '5 minutes'
GROUP BY user_id, ip_address
HAVING COUNT(*) >= 5;
```

#### Suspicious Login Patterns
```sql
-- SIEM Rule: Geographically Impossible Travel
SELECT DISTINCT a1.user_id, a1.ip_address as ip1, a2.ip_address as ip2,
       a1.timestamp as time1, a2.timestamp as time2
FROM authentication_logs a1
JOIN authentication_logs a2 ON a1.user_id = a2.user_id
WHERE a1.result = 'success' AND a2.result = 'success'
  AND a2.timestamp > a1.timestamp
  AND a2.timestamp < a1.timestamp + INTERVAL '1 hour'
  AND geoip_distance(a1.ip_address, a2.ip_address) > 1000; -- km
```

### Data Access Monitoring

#### Bulk Data Access
```sql
-- SIEM Rule: Unusual Data Access Volume
SELECT user_id, COUNT(*) as access_count
FROM data_access_logs
WHERE timestamp > NOW() - INTERVAL '1 hour'
  AND action IN ('read', 'export')
GROUP BY user_id
HAVING COUNT(*) > 100;
```

#### After-Hours Access
```sql
-- SIEM Rule: After-Hours Data Access
SELECT user_id, ip_address, action, resource
FROM data_access_logs
WHERE HOUR(timestamp) NOT BETWEEN 8 AND 18
  AND DAYOFWEEK(timestamp) BETWEEN 2 AND 6
  AND action IN ('read', 'write', 'delete', 'export');
```

### System Security Monitoring

#### Process Monitoring
```bash
# Auditd rules for process monitoring
# /etc/audit/rules.d/process.rules

# Monitor privilege escalation
-a always,exit -F arch=b64 -S execve -F euid=0 -F auid!=0 -k privilege_escalation

# Monitor sensitive file access
-w /etc/passwd -p wa -k user_modification
-w /etc/shadow -p wa -k password_modification
-w /etc/sudoers -p wa -k privilege_modification
```

#### File Integrity Monitoring
```bash
# AIDE configuration for file integrity
# /etc/aide/aide.conf

# Critical system files
/bin                    NORMAL
/sbin                   NORMAL
/usr/bin                NORMAL
/usr/sbin               NORMAL
/etc                    NORMAL

# Application files
/opt/passq              NORMAL
/var/www/passq          NORMAL
```

### Network Security Monitoring

#### DDoS Detection
```bash
# Network traffic analysis script
#!/bin/bash

# Monitor connection rates
CONN_RATE=$(netstat -an | grep :443 | grep SYN_RECV | wc -l)
if [ $CONN_RATE -gt 1000 ]; then
    logger -p local0.warning "High connection rate detected: $CONN_RATE"
    # Trigger DDoS mitigation
fi
```

#### Port Scan Detection
```yaml
# Suricata rule for port scanning
alert tcp any any -> $HOME_NET any (msg:"Port Scan Detected"; 
    flags:S; threshold: type both, track by_src, count 10, seconds 60; 
    sid:1000001; rev:1;)
```

## Alert Management

### Alert Severity Levels

#### Critical (P0) - Immediate Response
- **Data Breach Indicators**: Unauthorized data access
- **System Compromise**: Malware detection, backdoors
- **Service Outage**: Complete system unavailability
- **Regulatory Violations**: Compliance breach detection

**Response Time**: 15 minutes
**Escalation**: Immediate to Security Team and Management

#### High (P1) - Urgent Response
- **Security Control Bypass**: Authentication failures
- **Suspicious Activity**: Unusual access patterns
- **Performance Degradation**: Significant service impact
- **Vulnerability Exploitation**: Active exploit attempts

**Response Time**: 1 hour
**Escalation**: Security Team within 30 minutes

#### Medium (P2) - Standard Response
- **Policy Violations**: Internal security policy breaches
- **Anomalous Behavior**: Deviation from baseline
- **Failed Security Events**: Blocked attacks
- **Configuration Issues**: Security misconfigurations

**Response Time**: 4 hours
**Escalation**: Security Team during business hours

#### Low (P3) - Informational
- **Routine Events**: Normal security operations
- **Maintenance Activities**: Scheduled security tasks
- **Baseline Updates**: Normal pattern changes
- **Compliance Reports**: Regular compliance checks

**Response Time**: 24 hours
**Escalation**: Security Team review

### Alert Correlation Rules

#### Multi-Stage Attack Detection
```sql
-- Correlation Rule: Reconnaissance + Exploitation
SELECT DISTINCT t1.source_ip, t1.timestamp as recon_time, 
       t2.timestamp as exploit_time
FROM threat_events t1
JOIN threat_events t2 ON t1.source_ip = t2.source_ip
WHERE t1.event_type = 'port_scan'
  AND t2.event_type = 'exploitation_attempt'
  AND t2.timestamp > t1.timestamp
  AND t2.timestamp < t1.timestamp + INTERVAL '24 hours';
```

#### Credential Stuffing Detection
```sql
-- Correlation Rule: Multiple Failed Logins Across Users
SELECT source_ip, COUNT(DISTINCT user_id) as affected_users,
       COUNT(*) as total_attempts
FROM authentication_logs
WHERE result = 'failure'
  AND timestamp > NOW() - INTERVAL '10 minutes'
GROUP BY source_ip
HAVING COUNT(DISTINCT user_id) >= 10
   AND COUNT(*) >= 50;
```

## Dashboards and Reporting

### Security Operations Dashboard

#### Real-Time Metrics
- **Active Alerts**: Current security alerts by severity
- **Authentication Status**: Login success/failure rates
- **System Health**: Service availability and performance
- **Threat Landscape**: Current threat indicators

#### Key Performance Indicators
```sql
-- Dashboard Query: Authentication Success Rate
SELECT 
    DATE(timestamp) as date,
    COUNT(CASE WHEN result = 'success' THEN 1 END) as successful_logins,
    COUNT(CASE WHEN result = 'failure' THEN 1 END) as failed_logins,
    ROUND(COUNT(CASE WHEN result = 'success' THEN 1 END) * 100.0 / COUNT(*), 2) as success_rate
FROM authentication_logs
WHERE timestamp >= CURDATE() - INTERVAL 30 DAY
GROUP BY DATE(timestamp)
ORDER BY date;
```

### Executive Security Dashboard

#### Monthly Security Metrics
- **Security Incidents**: Total incidents by category
- **Response Times**: Mean time to detection/response
- **Compliance Status**: Regulatory compliance metrics
- **Risk Assessment**: Current risk posture

#### Trend Analysis
```sql
-- Executive Report: Monthly Security Trends
SELECT 
    YEAR(timestamp) as year,
    MONTH(timestamp) as month,
    COUNT(*) as total_incidents,
    COUNT(CASE WHEN severity = 'critical' THEN 1 END) as critical_incidents,
    AVG(TIMESTAMPDIFF(MINUTE, detected_at, resolved_at)) as avg_resolution_time
FROM security_incidents
WHERE timestamp >= CURDATE() - INTERVAL 12 MONTH
GROUP BY YEAR(timestamp), MONTH(timestamp)
ORDER BY year, month;
```

## Compliance Monitoring

### GDPR Compliance Monitoring

#### Data Processing Activities
```sql
-- Monitor data processing for GDPR compliance
SELECT 
    user_id,
    data_type,
    processing_purpose,
    legal_basis,
    timestamp
FROM data_processing_logs
WHERE timestamp > NOW() - INTERVAL '24 hours'
  AND data_type IN ('personal_data', 'sensitive_data');
```

#### Data Subject Rights
```sql
-- Track data subject rights requests
SELECT 
    request_type,
    COUNT(*) as request_count,
    AVG(TIMESTAMPDIFF(DAY, request_date, completion_date)) as avg_response_time
FROM data_subject_requests
WHERE request_date >= CURDATE() - INTERVAL 30 DAY
GROUP BY request_type;
```

### SOC2 Compliance Monitoring

#### Access Control Monitoring
```sql
-- SOC2 Trust Service Criteria monitoring
SELECT 
    control_id,
    control_description,
    test_result,
    test_date,
    next_test_date
FROM soc2_controls
WHERE test_date >= CURDATE() - INTERVAL 90 DAY
ORDER BY control_id;
```

## Automated Response Actions

### Threat Response Automation

#### Account Lockout
```python
# Automated account lockout for suspicious activity
def auto_lockout_account(user_id, reason):
    # Lock account
    db.execute(
        "UPDATE users SET account_locked = true, locked_reason = %s WHERE id = %s",
        (reason, user_id)
    )
    
    # Log security event
    security_log.warning(
        "Account automatically locked",
        user_id=user_id,
        reason=reason,
        timestamp=datetime.utcnow()
    )
    
    # Notify security team
    send_alert(
        severity="high",
        message=f"Account {user_id} locked due to {reason}"
    )
```

#### IP Blocking
```bash
#!/bin/bash
# Automated IP blocking script

BLOCK_IP=$1
REASON=$2

# Add to firewall
iptables -A INPUT -s $BLOCK_IP -j DROP

# Log the action
logger -p local0.warning "IP $BLOCK_IP blocked automatically: $REASON"

# Notify security team
echo "IP $BLOCK_IP blocked: $REASON" | mail -s "Automated IP Block" security@passq.com
```

### Incident Escalation

#### Automated Escalation Rules
```python
# Escalation logic based on alert severity and time
def escalate_alert(alert_id, current_severity, time_open):
    escalation_rules = {
        'critical': {'initial': 15, 'escalate_after': 30},  # minutes
        'high': {'initial': 60, 'escalate_after': 120},
        'medium': {'initial': 240, 'escalate_after': 480},
        'low': {'initial': 1440, 'escalate_after': 2880}  # minutes
    }
    
    rule = escalation_rules.get(current_severity)
    if time_open > rule['escalate_after']:
        # Escalate to next level
        escalate_to_management(alert_id)
        
    elif time_open > rule['initial']:
        # Send reminder to assigned analyst
        send_reminder(alert_id)
```

## Performance and Tuning

### Monitoring Performance Metrics

#### SIEM Performance
- **Ingestion Rate**: Events per second processed
- **Search Performance**: Query response times
- **Storage Utilization**: Disk usage and retention
- **Alert Volume**: Alerts generated per day

#### Optimization Strategies
```bash
# Elasticsearch optimization for SIEM
# /etc/elasticsearch/elasticsearch.yml

# Memory allocation
bootstrap.memory_lock: true

# Index optimization
index.refresh_interval: 30s
index.number_of_shards: 3
index.number_of_replicas: 1

# Query optimization
search.max_buckets: 65536
indices.query.bool.max_clause_count: 10000
```

### Alert Tuning

#### False Positive Reduction
```sql
-- Analyze false positive rates
SELECT 
    alert_rule,
    COUNT(*) as total_alerts,
    COUNT(CASE WHEN false_positive = true THEN 1 END) as false_positives,
    ROUND(COUNT(CASE WHEN false_positive = true THEN 1 END) * 100.0 / COUNT(*), 2) as fp_rate
FROM security_alerts
WHERE created_at >= CURDATE() - INTERVAL 30 DAY
GROUP BY alert_rule
HAVING fp_rate > 10
ORDER BY fp_rate DESC;
```

#### Rule Optimization
```python
# Dynamic threshold adjustment based on baseline
def adjust_alert_threshold(rule_id, baseline_data):
    current_threshold = get_rule_threshold(rule_id)
    baseline_mean = statistics.mean(baseline_data)
    baseline_std = statistics.stdev(baseline_data)
    
    # Set threshold at 3 standard deviations above mean
    new_threshold = baseline_mean + (3 * baseline_std)
    
    if abs(new_threshold - current_threshold) > (current_threshold * 0.1):
        update_rule_threshold(rule_id, new_threshold)
        log_threshold_change(rule_id, current_threshold, new_threshold)
```

## Maintenance and Updates

### Regular Maintenance Tasks

#### Daily Tasks
- Review critical and high-severity alerts
- Verify SIEM data ingestion rates
- Check system health dashboards
- Update threat intelligence feeds

#### Weekly Tasks
- Analyze alert trends and patterns
- Review and tune alert rules
- Update security signatures
- Conduct log retention cleanup

#### Monthly Tasks
- Generate security metrics reports
- Review and update monitoring procedures
- Conduct alert rule effectiveness analysis
- Update baseline behavior models

#### Quarterly Tasks
- Comprehensive monitoring system review
- Update threat hunting procedures
- Review and update escalation procedures
- Conduct monitoring system penetration testing

### System Updates

#### SIEM Platform Updates
```bash
#!/bin/bash
# SIEM update procedure

# Backup current configuration
cp -r /opt/splunk/etc /backup/splunk-config-$(date +%Y%m%d)

# Update Splunk
sudo /opt/splunk/bin/splunk stop
sudo /opt/splunk/bin/splunk start --accept-license

# Verify update
/opt/splunk/bin/splunk version
```

#### Signature Updates
```bash
#!/bin/bash
# Automated signature updates

# Update Suricata rules
suricata-update
sudo systemctl reload suricata

# Update ClamAV signatures
freshclam
sudo systemctl reload clamav-daemon

# Update YARA rules
cd /opt/yara-rules
git pull origin main
sudo systemctl reload yara-scanner
```

---

**Document Control**
- **Version**: 1.0
- **Created**: 2025-01-27
- **Last Modified**: 2025-01-27
- **Next Review**: 2025-04-27
- **Classification**: Confidential
- **Owner**: Security Operations Team
- **Approved By**: CISO, Operations Manager