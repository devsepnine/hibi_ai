---
name: git-repos-syncer
description: "여러 Git 저장소의 상태를 확인하거나 fast-forward로 갱신한다."
---

# 로컬 Git 저장소 확인하고 갱신하기

여러 로컬 저장소를 한 번에 다룰 때 이 스킬을 사용한다. 상태 확인 요청은 결과 보고까지 진행한다. 갱신 요청에는 사용자가 정한 범위에서 선택한 로컬 체크아웃을 fast-forward로 갱신한다.

## 체크아웃 확인하고 비교하기

1. 사용자가 지정한 경로나 상위 디렉터리를 사용한다. 해당 범위 안에서 저장소를 찾고 같은 체크아웃 경로는 한 번만 처리한다. 각 저장소에서 사용할 브랜치와 체크아웃을 확인한다. 사용자가 다른 대상을 지정하지 않았으면 현재 브랜치와 그 브랜치에 설정된 upstream을 사용한다. 다른 로컬 브랜치를 대상으로 하면 해당 브랜치를 사용 중인 체크아웃을 찾는다.
2. 각 체크아웃의 상태를 살핀다. 스테이징한 변경, 스테이징하지 않은 변경, 추적되지 않는 파일과 진행 중인 Git 작업을 확인한다. detached checkout, 사용할 upstream이 없는 브랜치, 체크아웃이 없는 브랜치는 갱신할 수 없는 이유를 알린다.
3. 저장된 ref로 상태를 비교할 때는 캐시된 정보라고 표시한다. 새로 확인하거나 갱신할 때는 선택한 원격 브랜치를 fetch한다. `remote`가 설정된 원격 이름인지, `remote_ref`가 그 원격의 전체 `refs/heads/` 경로인지, `tracking_ref`가 대응하는 전체 `refs/remotes/` 경로인지 확인한다. 원격 `HEAD`가 심볼릭 ref라면 실제 브랜치를 확인한 뒤 fetch한다. fetch에 실패하면 오류를 알리고 해당 항목을 갱신 대상에서 뺀다.

   ```bash
   git -C "$checkout" fetch --no-tags --no-prune --no-prune-tags \
     --no-recurse-submodules --no-write-fetch-head --refmap= -- \
     "$remote" "$remote_ref:$tracking_ref"
   ```

4. 로컬과 upstream의 끝 커밋 ID를 확인한다. `git rev-list --left-right --count`에서 왼쪽에 로컬, 오른쪽에 upstream을 두고 비교한다. 끝 커밋이 같으면 최신, 원격에만 커밋이 있으면 뒤처짐, 로컬에만 있으면 앞섬, 양쪽에 각각 있으면 분기로 표시한다. 조상 관계를 확인할 수 없으면 미확인으로 표시한다. 체크아웃의 변경 여부는 브랜치 비교 결과와 따로 알린다.

## 선택한 체크아웃 fast-forward 하기

1. 갱신할 수 있는 체크아웃과 건너뛸 이유를 보여준다. 사용자가 요청한 범위에서만 적용한다. 체크아웃에 변경이나 진행 중인 Git 작업이 없고, fetch한 upstream보다 뒤처져 있을 때 갱신할 수 있다. 로컬 끝 커밋이 대상 커밋의 조상인지 확인한다.
2. 각 체크아웃을 갱신하기 직전에 경로, 브랜치, 로컬 끝 커밋, upstream과 작업 상태가 미리보기와 일치하는지 확인한다. 달라졌으면 해당 체크아웃의 갱신을 보류하고 미리보기를 다시 확인한다. `target_commit`에는 기록한 upstream 커밋을 사용한다.

   ```bash
   git -C "$checkout" -c submodule.recurse=false merge \
     --ff-only --no-autostash --no-squash --no-overwrite-ignore -- "$target_commit"
   ```

3. 브랜치, 커밋과 체크아웃 상태를 다시 읽는다. 요청한 브랜치가 계획한 커밋에 도달한 것을 확인하면 갱신됐다고 알린다. Git이 무시한 파일과 충돌한다고 알리면 해당 체크아웃의 갱신을 보류하고 경로를 보고한다. Git 명령이 실패하거나 상태가 예상과 다르면 확인한 내용을 알리고 독립된 체크아웃은 계속 처리한다. 경로별 브랜치, upstream과 결과를 요약한다.
