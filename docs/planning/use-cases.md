# PAII Use Cases

> Real-world scenarios showing how PAII solves actual problems.

---

## Work Workflows

*These examples happen to be from an SRE context, but the patterns apply to any domain.*

### Incident Response

#### Scenario

It's 2 AM. PagerDuty pages you. A service is returning 500 errors.

#### Without PAII

1. Acknowledge in PagerDuty (manual)
2. Create Slack incident channel (manual)
3. Create Jira incident ticket (manual)
4. Find relevant runbooks (search Confluence)
5. Check similar past incidents (search Jira/history)
6. Execute runbook steps (manual)
7. Update stakeholders (manual)
8. Write post-mortem (manual, often forgotten)

**Problems:**
- Context switching between tools
- Forgetting steps under stress
- Losing knowledge when incident ends
- Post-mortems delayed or skipped

#### With PAII

```
You: "incident: payments-api is returning 500s in production"

Claude (using incident skill):

🚨 INCIDENT DECLARED: payments-api-500s

  PagerDuty: Acknowledged incident #12345 ✓
  Slack: Created #inc-payments-api-500s, invited on-call team ✓
  Jira: Created INC-789 (linked to PagerDuty) ✓

  📊 SIMILAR INCIDENTS (from history):

  1. INC-456 (2 weeks ago) - payments-api-timeout
     Root cause: Database connection pool exhaustion
     Resolution: Increased pool size from 50 to 100

  2. INC-234 (1 month ago) - payments-api-errors
     Root cause: Redis cache expired
     Resolution: Extended TTL on critical keys

  📋 SUGGESTED RUNBOOK: payments-api-health-check.md

  Ready for guided troubleshooting?

  Similar incident INC-456 was caused by connection pool exhaustion.
  Should I check database connections first?

You: "yes, check database"

Claude:
  Running: kubectl exec -it payments-api-pod -- python manage.py dbstats

  📊 Database Connection Status:
    Active connections: 98/100
    Waiting queries: 47

  ⚠️ Connection pool nearly exhausted (98% usage)
  This matches the pattern from INC-456.

  Recommended action: Scale database pool or restart pods.
  Should I proceed?
```

#### Plugins Involved

| Plugin | Role |
|--------|------|
| `incident` | Skill: orchestrates incident workflow |
| `pagerduty` | Integration: acknowledge, escalate |
| `slack` | Integration: create channel, notify |
| `jira` | Integration: create ticket |
| `history` | Memory: find similar incidents |

---

### On-Call Handoff

#### Scenario

Your on-call shift is ending. You need to brief the next person.

#### Without PAII

1. Try to remember everything that happened
2. Write up notes in Slack or email
3. Forget half the context
4. Next person starts cold

#### With PAII

```
You: "generate on-call handoff for the past 24 hours"

Claude (using oncall skill):

📋 ON-CALL HANDOFF SUMMARY
Period: 2026-01-01 09:00 to 2026-01-02 09:00
Engineer: Scott Idler

INCIDENTS HANDLED (2):

1. INC-789: payments-api-500s
   - Status: RESOLVED
   - Root cause: Database connection pool exhaustion
   - Resolution: Increased pool size to 150
   - Post-mortem: Scheduled for 2026-01-03
   - Slack thread: #inc-payments-api-500s

2. INC-790: auth-service-latency
   - Status: MONITORING
   - Current state: Latency reduced from 2s to 200ms
   - Action taken: Restarted auth pods
   - Watch for: Any latency spikes in next 4 hours

ALERTS (suppressed):

- Low disk warning on logs-storage-03 (false positive, disk expanded)
- Memory alert on cache-cluster (autoscaling handled it)

ONGOING WORK:

- Deploy pending: payments-api v2.3.4 (waiting for QA signoff)
- Maintenance window: 2026-01-03 02:00-04:00 for database upgrades

NOTES FOR NEXT SHIFT:

- Keep an eye on payments-api after the pool change
- Cache cluster has been autoscaling frequently - might need investigation

---

Shall I post this to #oncall-handoff?
```

#### Plugins Involved

| Plugin | Role |
|--------|------|
| `oncall` | Skill: generates handoff reports |
| `history` | Memory: retrieves session history |
| `pagerduty` | Integration: incident data |
| `slack` | Integration: post report |

---

### Runbook Execution

#### Scenario

You need to execute a database failover runbook.

#### Without PAII

1. Find the runbook in Confluence
2. Read through the steps
3. Execute each step manually
4. Document what you did
5. Hope you didn't miss anything

#### With PAII

```
You: "execute database failover runbook for primary-postgres"

Claude (using runbook skill):

📋 RUNBOOK: database-failover.md
Target: primary-postgres

⚠️ This runbook has 12 steps and affects production.
Estimated duration: 15-30 minutes
Last executed: 2025-11-15 (by Alice)

PRE-FLIGHT CHECKS:

  ✓ Replica lag: 0.2s (acceptable)
  ✓ Disk space: 45% used (OK)
  ✓ Active connections: 234 (will be migrated)
  ⚠️ Maintenance window: Not currently in window

Proceed with failover? (This will require approval for step 5 and step 9)

You: "yes, proceed"

Claude:

STEP 1/12: Enable read-only mode on primary
  Command: psql -c "ALTER SYSTEM SET default_transaction_read_only = on;"
  Executing...
  ✓ Complete

STEP 2/12: Wait for active transactions to complete
  Checking active transactions...
  Active: 12 -> 8 -> 3 -> 0
  ✓ All transactions complete

STEP 3/12: Verify replica is caught up
  Primary LSN: 0/5A000060
  Replica LSN: 0/5A000060
  ✓ Replica caught up

[... continues through all steps ...]

STEP 12/12: Update monitoring dashboards
  ✓ Grafana annotations added
  ✓ PagerDuty updated

═══════════════════════════════════════

✅ FAILOVER COMPLETE

  New primary: replica-postgres-01
  Duration: 18 minutes
  Connections migrated: 234
  Data loss: None

📝 Execution recorded to history/runbooks/2026-01/

Shall I create a Jira ticket documenting this failover?
```

#### Plugins Involved

| Plugin | Role |
|--------|------|
| `runbook` | Skill: step-by-step execution |
| `history` | Memory: record execution |
| `jira` | Integration: create documentation |

---

## Team Management

### Sprint Status Summary

#### Scenario

You need to update leadership on sprint progress.

```
You: "summarize sprint progress for Team Infra"

Claude (using jira skill):

📊 SPRINT SUMMARY: Team Infra - Sprint 23

Period: 2025-12-16 to 2025-12-30
Capacity: 5 engineers, 40 story points

PROGRESS:
  ▓▓▓▓▓▓▓▓▓▓▓▓░░░░ 75% complete (30/40 points)

  Completed: 12 tickets
  In Progress: 5 tickets
  Remaining: 3 tickets

BY EPIC:

  Database Migration (INFRA-100)
    ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ 100% (16/16 points)
    All tickets complete ✓

  Kubernetes Upgrade (INFRA-101)
    ▓▓▓▓▓▓▓▓▓▓░░░░░░ 60% (9/15 points)
    Blocked: Waiting for security review

  Monitoring Improvements (INFRA-102)
    ▓▓▓▓▓▓▓░░░░░░░░░ 55% (5/9 points)
    On track

BLOCKERS:

  1. INFRA-234: K8s upgrade security review
     Waiting on: Security team
     Days blocked: 3
     Escalated: No

RISKS:

  - Kubernetes upgrade may slip if security review not completed by EOD Friday

RECOMMENDATION:

  Sprint is at risk. Suggest escalating security review to unblock K8s work.

---

Shall I post this to #team-infra-status?
```

---

### Slack Channel Summarization

#### Scenario

You've been in meetings all day and need to catch up on a busy channel.

```
You: "summarize #platform-engineering for the past 8 hours"

Claude (using slack skill):

📢 CHANNEL SUMMARY: #platform-engineering
Period: Last 8 hours (127 messages)

KEY DISCUSSIONS:

1. 🔥 Production Issue: Cache invalidation bug
   - Reported by: @alice at 10:23
   - Status: Resolved at 14:15
   - Resolution: Fixed race condition in cache layer
   - Participants: Alice, Bob, Charlie
   - Thread: https://slack.com/archives/...

2. 📋 RFC Review: New deployment pipeline
   - Started by: @david at 11:00
   - Status: Open for feedback until Friday
   - Key concerns: Rollback time, canary percentage
   - Document: https://docs.company.com/rfc-042

3. 💬 General Discussion: On-call rotation changes
   - Proposal: Switch to 1-week rotations
   - Sentiment: Mixed (5 in favor, 3 against)
   - Action: @eve creating survey

MENTIONS OF YOU:

  @scott mentioned by @alice at 10:45:
  "Can you review the cache fix when you're free?"

  @scott mentioned by @frank at 13:22:
  "Need your approval on the infra budget PR"

ACTION ITEMS FOR YOU:

1. Review cache fix PR (from Alice)
2. Approve infra budget PR (from Frank)

---

Should I open those PRs for review?
```

---

## Personal Productivity

### Spanish Learning

#### Scenario

You're learning Spanish and want daily practice.

```
You: "let's practice Spanish conversation - intermediate level"

Claude (using spanish skill):

🇪🇸 ¡Hola! Vamos a practicar español.

Today's theme: "En el restaurante" (At the restaurant)

---

SCENARIO:
You're at a restaurant in Madrid. The waiter approaches your table.

Camarero: "¡Buenas tardes! ¿Qué desean tomar?"

(What would you like to say? Respond in Spanish.)

You: "Buenas tardes. Quisiera ver el menu, por favor."

Claude:

✓ Great sentence structure!

FEEDBACK:
- "Quisiera" is perfect - polite conditional form
- Minor spelling: "menú" (with accent)

Camarero: "Por supuesto. Aquí tiene el menú. ¿Algo para beber mientras mira?"

VOCABULARY HINT:
- "mientras" = while
- "mira" = you look (informal form of "mirar")

(Continue the conversation...)

You: "Si, una agua con gas, por favor."

Claude:

✓ Good!

FEEDBACK:
- In Spanish, "agua" uses "un" despite being feminine: "un agua con gas"
- This is because "agua" starts with stressed "a"

Corrected: "Sí, un agua con gas, por favor."

---

SESSION STATS:
  Duration: 15 minutes
  Sentences: 8
  Vocabulary learned: 12 words
  Grammar points: 2

Shall I save these notes to your learning history?
```

---

### Article Writing

#### Scenario

You're writing a technical blog post.

```
You: "help me write an article about chaos engineering"

Claude (using writing skill):

📝 ARTICLE PLANNING: Chaos Engineering

Let's start with structure. Based on your previous articles, you prefer:
- Practical, hands-on approach
- Real-world examples
- 1500-2500 word range

SUGGESTED OUTLINE:

1. Introduction (200 words)
   - Hook: "The last outage you'll never have to debug manually"
   - What is chaos engineering?
   - Why engineers should care

2. Principles (300 words)
   - Steady state hypothesis
   - Vary real-world events
   - Run in production
   - Minimize blast radius

3. Getting Started (400 words)
   - Picking your first experiment
   - Tools: Chaos Monkey, Litmus, Gremlin
   - Example: Killing a pod

4. Real-World Example (500 words)
   - Walk through a chaos experiment
   - What we learned
   - How it prevented a real outage

5. Building a Chaos Practice (300 words)
   - Game days
   - Automation
   - Cultural buy-in

6. Conclusion (100 words)
   - Key takeaways
   - Next steps

---

Does this outline work? Would you like to:
1. Start writing section by section
2. Modify the outline
3. Research specific topics first
```

---

## Meta/Dogfooding

### Improving PAII

#### Scenario

PAII helps improve itself.

```
You: "analyze my PAII usage and suggest improvements"

Claude (using improve skill):

📊 PAII USAGE ANALYSIS
Period: Last 30 days

PLUGIN USAGE:

  incident     ████████████████████ 45 invocations
  history      ████████████████░░░░ 38 invocations
  jira         ██████████░░░░░░░░░░ 22 invocations
  slack        ████████░░░░░░░░░░░░ 18 invocations
  runbook      ██████░░░░░░░░░░░░░░ 14 invocations
  spanish      ████░░░░░░░░░░░░░░░░ 9 invocations

PATTERNS IDENTIFIED:

1. Incident Response Gap
   - You frequently ask about "similar incidents"
   - Current: Manual search through history
   - Suggestion: Create "incident-similarity" plugin with ML-based matching

2. Missing Integration
   - 12 mentions of "Datadog" with no plugin installed
   - You're manually checking dashboards
   - Suggestion: Install or create Datadog integration plugin

3. Workflow Friction
   - Post-incident, you often manually create post-mortem docs
   - Suggestion: Add auto-postmortem workflow to incident plugin

4. Unused Capabilities
   - `runbook` plugin supports dry-run mode
   - You've never used it
   - Consider: Use dry-run for critical runbooks

RECOMMENDATIONS:

1. 🔴 HIGH: Create Datadog integration (you mention it daily)
2. 🟡 MEDIUM: Add post-mortem automation to incident plugin
3. 🟢 LOW: Explore incident similarity matching

---

Would you like me to scaffold the Datadog plugin?

You: "yes, create the Datadog plugin"

Claude:

📦 CREATING PLUGIN: datadog

paii plugin new datadog --type integration --language python

Created:
  datadog/
  ├── plugin.toml
  ├── pyproject.toml
  └── src/
      └── plugin.py

I've pre-filled the manifest based on typical Datadog integration needs:

[provides]
integration = { contract = "IntegrationProvider", service = "datadog" }

[config]
api_key = { type = "string", required = true, env = "DD_API_KEY", secret = true }
app_key = { type = "string", required = true, env = "DD_APP_KEY", secret = true }
site = { type = "string", default = "datadoghq.com" }

Actions to implement:
- query_metrics
- get_dashboard
- list_monitors
- create_monitor

Shall I implement the basic API client?
```

---

## Summary

| Use Case | Plugins Used | Key Benefit |
|----------|--------------|-------------|
| Incident Response | incident, pagerduty, slack, jira, history | Coordinated multi-tool workflow |
| On-Call Handoff | oncall, history, pagerduty, slack | Context preservation |
| Runbook Execution | runbook, history, jira | Guided, documented execution |
| Sprint Status | jira, slack | Automated reporting |
| Slack Summary | slack, history | Async team context |
| Spanish Learning | spanish, history | Personalized practice |
| Article Writing | writing, history | Structured composition |
| Self-Improvement | improve, history | Continuous optimization |

---

## Related Documents

- [plugins.md](plugins.md) — How to build plugins
- [architecture.md](architecture.md) — System design
- [vision.md](vision.md) — Philosophy and goals

