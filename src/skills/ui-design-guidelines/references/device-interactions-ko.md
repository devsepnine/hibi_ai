# 기기별 상호작용

## 휴대 기기와 펜 입력

폰에서 자주 쓰는 기능은 손이 닿는 곳에 두고 키보드가 열려도 누를 영역이 구분되게 한다. 짧은 값을 반복 입력할 때는 데이터가 뒷받침하면 최근 값이나 선택기를 제공한다. 키패드와 터치를 함께 지원하면 소프트 키 문구를 해당 물리 키 가까이에 둔다. 2003년 *Palm OS User Interface Guidelines*, 2010년 Nokia의 *Series 40 Touch and Type UI Style Guide*, 2018년 *One UI Design Guidelines*를 참고했다.

필기나 스캔한 글자를 인식해 중요한 명령에 사용할 때는 해석된 값을 먼저 보여 준다. 해당 값만 고칠 경로를 두고 다른 입력은 보존한다. 펜 제스처는 컨트롤을 통해 찾을 수 있는 명령을 빠르게 실행하는 데 쓸 수 있다. 1996년 *Newton 2.0 User Interface Guidelines*와 1991년 *PenPoint UI Design Guidelines*를 참고했다.

## 시계와 연동 휴대폰

시계는 잠깐 보아도 당장 궁금한 점과 간단한 다음 행동을 파악할 수 있어야 한다. 기기 간 이어서 하기를 지원하면 긴 문장 작성이나 비교는 휴대폰에서 한다. 현장 점검이라면 시계에 인식된 장비 번호를 보여 주고 휴대폰을 열었을 때 같은 기록임을 확인할 단서를 둔다. 로컬 기록, 전송 대기, 수신 확인, 전송 중단은 실제 저장·동기화 보장에 맞춰 설계한다. 동작이 정해지지 않았다면 설계에 필요한 전제임을 표시한다.

2014년 *Android Wear Notification and Application Patterns*와 2015년 초안인 Samsung의 *The Gear UI Design Guidelines*를 참고했다. 베젤이나 하드웨어 버튼에 기능을 배정하기 전에 실제 기기의 구성을 확인한다.

## 공용 화면과 공간 입력

- 테이블에서는 정보를 읽는 사람의 위치에 맞춰 레이블을 배치하거나 회전한다. 한 사람이 객체를 움직일 때 다른 사람이 다른 곳에서 할 수 있는 작업을 정한다. 두 사람이 서로 다른 객체를 조작하는 경우와 같은 객체에 손을 뻗는 경우를 확인한다. 출처는 2011년 *Microsoft Surface 2.0 Design and Interaction Guide*의 “Orientation & Layout”이다.
- 카메라 제스처 입력은 사람이 감지된 상태, 제어권을 얻은 상태, 대상을 가리킨 상태, 명령 실행을 구분한다. 현재 조작자와 추적이 끊겼을 때 남는 선택을 보여 준다. 쉬거나 제어권을 놓을 방법을 제공한다. 출처는 *Kinect Human Interface Guidelines* 2.0의 “Engagement”다.
- 헤드셋에서는 가리킨 대상과 실제 실행 시점에 구별되는 단서를 준다. 장면을 바꿀 때 가능하면 고정된 기준점을 유지하고 사용자가 이동을 시작하게 한다. 편안하다고 판단하려면 대상 기기에서 체험한다. 출처는 2016년 *Designing for Google Cardboard*의 “Feedback”과 “Physiological considerations”다.
