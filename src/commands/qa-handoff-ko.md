---
description: Summarize recent git history into a QA handoff document with a plain-language change summary plus an executable QA checklist for non-developers.
argument-hint: "[range]"
allowed-tools: Read, Write, Grep, Glob, Bash
model: sonnet
effort: high
---

# QA Handoff

완료된 개발 내역을 QA·기획·CS에 넘기기 위한 얇은 진입점이다. 커밋 범위를
확정한다. `$ARGUMENTS`가 있으면 그것이 범위이며 `v1.14.0..HEAD`, 브랜치명, "지난주" 등이 가능하다.
방금 완료한 작업이면 확인된 해당 커밋을 쓴다. 그 외에는 스킬 기본값인
"HEAD에서 도달 가능한 최신 태그부터 HEAD"를 쓴다. 그다음 커밋 제목이
아니라 diff를 읽어서 두 단으로 된 문서를 만든다. 구성은 제품 용어로 쓴 변경 요약,
그리고 항목마다 전제·재현 절차·기대 결과를 갖춘 QA 체크리스트이다.
변경 전후를 비교하고 내부 유지보수 내역과 체크리스트 우선순위 표시는 제외한다.
API 계약이 바뀌면 어떤 값이 변경됐고 제품의 어느 화면이나 동작에서 쓰이는지 설명한다.

호출 방법: 기능·릴리즈 브랜치 작업이 끝난 뒤, 또는 "무엇이 바뀌었고 무엇을
테스트해야 하나"라는 질문이 나올 때 실행한다. 사용자가 지정하지 않았다면 결과를
어디에 남길지 묻는다. 선택지는 저장소 마크다운 파일, 대화창 출력, Jira, Confluence 게시다.

필수 원칙: 기대 결과를 추측하지 않는다. diff로 분류할 수 없는 변경은 개발자 확인 사항에
넣는다. 외부에서 관찰할 수 없는 변경은 문서에서 제외하되, 보안과 데이터 무결성의
비정상 경로 검증은 유지한다.

**전체 방법론과 문서 템플릿은 `qa-handoff` skill을 source of truth로 삼아 따른다.**
