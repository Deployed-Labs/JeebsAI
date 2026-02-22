SELECT key, length(value) FROM jeebs_store WHERE key LIKE '%peci%' OR key LIKE '%peaci%';
.headers on
.mode column

SELECT id, substr(message,1,200) as msg FROM system_logs WHERE message LIKE '%peci%' OR message LIKE '%peaci%' LIMIT 50;
