SELECT COUNT(*) FROM user_sessions WHERE username='peaci';
SELECT COUNT(*) FROM user_sessions WHERE username='peci';
SELECT COUNT(*) FROM jeebs_store WHERE key LIKE '%peci%' OR key LIKE '%peaci%';
SELECT COUNT(*) FROM system_logs WHERE message LIKE '%peci%' OR message LIKE '%peaci%';
