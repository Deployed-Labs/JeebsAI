import sqlite3, json, time, uuid, random, re, urllib.request, html
from datetime import datetime, timezone

DB="/root/JeebsAI/jeebs.db"
conn=sqlite3.connect(DB)
cur=conn.cursor()

def now():
    return datetime.now(timezone.utc).isoformat()

run_id=str(uuid.uuid4())
session_id=str(uuid.uuid4())
run_key=f"deeplearn_run:{run_id}"
session_key=f"learnsession:{session_id}"

# minimal DeepLearningSession structure
session={
    "id": session_id,
    "topic": "internet research (ad-hoc)",
    "depth_level": 1,
    "subtopics": ["web scraping","knowledge extraction"],
    "learned_facts": [],
    "questions_answered": [],
    "practice_problems": [],
    "connections_made": [],
    "started_at": now(),
    "last_studied": now(),
    "study_hours": 0.0,
    "confidence": 0.2,
    "status": "novice",
}

meta={"id": run_id, "status": "starting", "progress_percent": 0.0, "history": []}

# write initial records
cur.execute("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)", (session_key, sqlite3.Binary(json.dumps(session).encode('utf-8'))))
cur.execute("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)", (run_key, sqlite3.Binary(json.dumps(meta).encode('utf-8'))))
conn.commit()
print('RUN_STARTED', run_id, session_id)

# seeds
try:
    with open('/root/JeebsAI/research_allowlist.txt') as f:
        seeds=[l.strip() for l in f if l.strip()]
except Exception:
    seeds=['https://en.wikipedia.org/wiki/Special:Random','https://arxiv.org']

minutes=10
end=time.time()+minutes*60
iter_count=0
while time.time()<end:
    iter_count+=1
    seed=random.choice(seeds)
    try:
        req=urllib.request.Request(seed, headers={'User-Agent':'JeebsAI-test-bot/1.0'})
        with urllib.request.urlopen(req, timeout=15) as r:
            body=r.read().decode('utf-8', errors='replace')
            text=re.sub('<script.*?</script>','',body, flags=re.S|re.I)
            text=re.sub('<[^>]+>','',text)
            snippet=html.unescape(text).strip()[:800].replace('\n',' ')
            source=seed
            fact={'fact': f"[auto-research:{source}] {snippet}", 'source': f'web:{source}', 'learned_at': now(), 'importance': 0.4, 'used_in_responses':0, 'related_concepts': []}
            # update session
            cur.execute('SELECT value FROM jeebs_store WHERE key=?',(session_key,))
            row=cur.fetchone()
            if row:
                s=json.loads(row[0])
            else:
                s=session
            s.setdefault('learned_facts',[]).append(fact)
            s['last_studied']=now()
            s['study_hours']=s.get('study_hours',0.0)+0.1
            cur.execute("UPDATE jeebs_store SET value=? WHERE key=?", (sqlite3.Binary(json.dumps(s).encode('utf-8')), session_key))
            # update run meta
            cur.execute('SELECT value FROM jeebs_store WHERE key=?',(run_key,))
            row=cur.fetchone()
            m=json.loads(row[0]) if row else meta
            pct=min(((time.time()- (end - minutes*60))/(minutes*60))*100, 99.0)
            m['progress_percent']=pct
            m['last_update']=now()
            m.setdefault('history',[]).append({'ts':now(),'seed':source,'progress_percent':pct})
            m['last_websites']=[source]
            m['last_learned_items']=[snippet[:300]]
            cur.execute("UPDATE jeebs_store SET value=? WHERE key=?", (sqlite3.Binary(json.dumps(m).encode('utf-8')), run_key))
            conn.commit()
            print('ITER', iter_count, 'seed', source, 'len', len(snippet))
    except Exception as e:
        print('ERR', e)
    time.sleep(5)

# finalize
m['status']='done'
m['progress_percent']=100.0
m['completed_at']=now()
cur.execute("UPDATE jeebs_store SET value=? WHERE key=?", (sqlite3.Binary(json.dumps(m).encode('utf-8')), run_key))
conn.commit()
print('RUN_DONE', run_id)
