---
description: Restate requirements, assess risks, and create step-by-step implementation plan. WAIT for user CONFIRM before touching any code.
allowed-tools: Task, Read, Grep, Glob
model: sonnet
effort: high
---

# Plan Command

이 커맨드는 built-in `Plan` 에이전트를 호출하여 코드 작성 전 종합 구현 계획을 수립한다.

## What This Command Does

1. **Restate Requirements** - 무엇을 만들어야 하는지 명확화한다
2. **Identify Risks** - 잠재적 이슈 및 차단 요인을 노출시킨다
3. **Create Step Plan** - 구현을 단계별로 분해한다
4. **Wait for Confirmation** - 진행 전 반드시 사용자 승인을 받아야 한다

## When to Use

다음 경우에 `/plan`을 사용한다:
- 새로운 피처를 시작할 때
- 중요한 아키텍처 변경을 할 때
- 복잡한 리팩토링 작업을 할 때
- 여러 파일/컴포넌트가 영향을 받을 때
- 요구사항이 불명확하거나 모호할 때

## How It Works

built-in `Plan` 에이전트는 다음을 수행한다:

1. **요청을 분석**하고 명확한 용어로 요구사항을 재진술한다
2. **단계별로 분해**하여 구체적이고 실행 가능한 단계로 만든다
3. 컴포넌트 간 **의존성을 식별**한다
4. **위험과 잠재적 차단 요인을 평가**한다
5. **복잡도를 추정**한다 (High/Medium/Low)
6. **계획을 제시**하고 사용자의 명시적 확인을 기다린다

## Example Usage

```
User: /plan I need to add real-time notifications when markets resolve

Agent (Plan):
# Implementation Plan: Real-Time Market Resolution Notifications

## Requirements Restatement
- Send notifications to users when markets they're watching resolve
- Support multiple notification channels (in-app, email, webhook)
- Ensure notifications are delivered reliably
- Include market outcome and user's position result

## Implementation Phases

### Phase 1: Database Schema
- Add notifications table with columns: id, user_id, market_id, type, status, created_at
- Add user_notification_preferences table for channel preferences
- Create indexes on user_id and market_id for performance

### Phase 2: Notification Service
- Create notification service in lib/notifications.ts
- Implement notification queue using BullMQ/Redis
- Add retry logic for failed deliveries
- Create notification templates

### Phase 3: Integration Points
- Hook into market resolution logic (when status changes to "resolved")
- Query all users with positions in market
- Enqueue notifications for each user

### Phase 4: Frontend Components
- Create NotificationBell component in header
- Add NotificationList modal
- Implement real-time updates via Supabase subscriptions
- Add notification preferences page

## Dependencies
- Redis (for queue)
- Email service (SendGrid/Resend)
- Supabase real-time subscriptions

## Risks
- HIGH: Email deliverability (SPF/DKIM required)
- MEDIUM: Performance with 1000+ users per market
- MEDIUM: Notification spam if markets resolve frequently
- LOW: Real-time subscription overhead

## Estimated Complexity: MEDIUM
- Backend: 4-6 hours
- Frontend: 3-4 hours
- Testing: 2-3 hours
- Total: 9-13 hours

**WAITING FOR CONFIRMATION**: Proceed with this plan? (yes/no/modify)
```

## Important Notes

**CRITICAL**: built-in `Plan` 에이전트는 사용자가 "yes" 또는 "proceed" 등 명시적인 긍정 응답으로 계획을 확인할 때까지 **어떤 코드도 작성하지 않는다**.

변경을 원할 경우 다음과 같이 응답한다:
- "modify: [your changes]"
- "different approach: [alternative]"
- "skip phase 2 and do phase 3 first"

## Integration with Other Commands

계획 수립 후:
- 테스트 주도 개발로 구현하려면 `/tdd` 사용
- 빌드 오류가 발생하면 `/build-and-fix` 사용
- 완료된 구현을 검토하려면 `/code-review` 사용

## Related Agents

이 커맨드는 harness built-in `Plan` 에이전트를 호출한다 (별도 커스텀 에이전트 파일 불필요).
