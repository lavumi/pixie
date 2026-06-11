# Pixie Next Steps

다음 작업 대화를 위한 프로젝트 현황과 개선 후보 정리 문서입니다.

## Current State

최근 완료된 주요 작업:

- `specs`에서 `hecs` 기반 ECS 구조로 이전
- `RenderWorldExtractor`를 도입해 ECS와 renderer 사이의 변환 책임 분리
- renderer가 현재 프레임의 draw list만 그리도록 변경
- `previous_atlases` 제거
- `Tile` 개념을 완전히 제거하고 `Sprite` 기반 렌더링으로 전환
- `Transform`에 radians 기반 `rotation` 추가
- sprite 모델 행렬에 `translation * rotation_z * scale` 적용
- `Velocity.angular`, `Force.torque`를 사용하는 기본 회전 물리 적분 추가
- Flappy Bird와 Physics Demo를 새 `Sprite`/`Transform` API로 마이그레이션

기준 커밋:

```text
45bfe6c ✨ Replace tiles with sprites and add rotation
a9e716c ♻️ Introduce render world extractor
2940cb8 🧹 Refresh docs and clean clippy warnings
```

마지막 검증 결과:

```text
cargo check
cargo clippy --all-targets
cargo test
cargo build --bin flappy --bin physics
```

모두 통과한 상태입니다.

## Recommended Priorities

### 1. Render Boundary 안정화

현재 구조:

```text
World + ResourceContainer
    -> RenderWorldExtractor
    -> RenderFrame
    -> RenderState
    -> GPUResourceManager
```

이 방향은 유지하는 것이 좋습니다. 다음 개선 후보:

- ~~`RenderFrame`과 extractor API의 소유권/수명 설계 재검토~~
- ~~atlas 이름 `String` clone 최소화~~
- sprite별 CPU buffer와 GPU instance buffer capacity 재사용 정책 정리
- ~~없는 atlas를 참조했을 때 panic 대신 명확한 오류 제공~~
- visibility, render layer, draw order를 어느 계층에서 처리할지 결정
- renderer와 extractor를 독립적으로 테스트할 수 있는 경계 마련

### 2. Renderer Error Handling

renderer와 resource manager에 `unwrap`/`panic` 기반 경로가 많이 남아 있습니다.

주요 대상:

- surface/adapter/device 생성
- texture decode 및 upload
- font atlas 생성
- pipeline/bind group/buffer/mesh lookup
- 중복 atlas/mesh/resource 등록

권장 진행 순서:

1. `GPUResourceManager` lookup API를 `Option`/`Result`로 전환
2. texture와 font loading 오류를 상위로 전달
3. `RenderState::new`, `init_resources`, `load_texture_atlas`에 오류 타입 적용
4. 마지막에 `Engine::start`와 WASM entrypoint까지 오류 흐름 연결

한 번에 전체를 바꾸기보다 작은 커밋으로 나누는 것이 좋습니다.

### 3. Rotation Physics 확장

현재 지원:

- sprite 중심 기준 렌더 회전
- angular velocity 적분
- `torque / mass` 기반 단순 angular acceleration

현재 미지원:

- 회전된 `BoxCollider`
- OBB 또는 SAT 충돌
- collider별 moment of inertia
- angular collision impulse
- friction과 angular damping
- text rotation
- configurable pivot/origin

현재 collision system은 `Transform.rotation`을 의도적으로 무시하는 axis-aligned 방식입니다. 다음 단계에서는 먼저 collider rotation을 지원할지, 렌더 rotation과 물리 rotation을 분리할지 결정해야 합니다.

### 4. Physics 구조 정리

현재 physics는 간단한 데모 수준이며 다음 부분이 커질 가능성이 높습니다.

- `Transform.size`와 collider 크기 사이의 관계가 암묵적임
- `RigidBody.mass`를 angular inertia처럼 사용하는 단순 모델
- Static/Kinematic/Dynamic 동작 규칙이 충분히 문서화되지 않음
- collision detection과 resolution이 한 파일에 밀집됨
- broad phase 없이 모든 collider 조합을 비교함

추천 분리:

```text
physics/
├── integration
├── collision_detection
├── collision_resolution
└── shapes
```

엔진 규모가 작을 동안은 직접 구현을 유지할 수 있지만, 정밀 물리가 목표라면 `rapier2d` 같은 검증된 라이브러리 도입도 비교할 필요가 있습니다.

### 5. Dispatcher 명칭과 역할

WASM에서는 single-thread dispatcher가 필요합니다. Native의 `MultiThreadedDispatcher`는 현재 이름과 달리 순차 실행 fallback입니다.

선택지:

- 현재 구현에 맞춰 `SequentialDispatcher`로 단순화
- native에서 실제 병렬 실행을 설계
- 시스템 scheduling 자체를 별도 모듈로 확장

`hecs::World`와 공유 `ResourceContainer`에 대한 mutable access 때문에 시스템 단위 병렬화는 단순히 rayon을 붙이는 작업이 아닙니다. 우선 이름과 문서를 현재 동작에 맞추는 것이 안전합니다.

### 6. Asset Lifecycle

현재 texture atlas는 문자열 이름을 기반으로 등록하고 조회합니다.

개선 후보:

- `String` 대신 typed handle 사용
- 중복 등록/누락 atlas 오류 처리
- renderer 초기화 전후 asset loading API 통일
- unload/reload 지원
- WASM과 native asset loading 흐름 통일
- texture atlas와 sprite sheet metadata 분리

### 7. Tests

현재 테스트는 다음 영역에 집중되어 있습니다.

- `ResourceContainer`
- gene evolution
- neural network layer
- sprite rotation matrix
- angular physics integration

추가 우선순위:

- `RenderWorldExtractor`가 sprite/text 데이터를 올바르게 생성하는지
- entity 삭제 후 text cache가 정리되는지
- 현재 프레임 draw list에 없는 sprite atlas가 렌더링되지 않는지
- animation UV 계산
- collision edge cases
- WASM target compile check

## Dependency Updates

확인 당시 주요 버전:

```text
wgpu 27.0.1   -> 최신 29.x 계열
winit 0.30.12 -> 0.31 beta 존재
hecs 0.10.5   -> 최신 0.11.x 계열
rand 0.8.5    -> 최신 0.10.x 계열
```

권장:

- `winit` beta 업데이트는 보류
- `hecs`, `rand`는 개별 커밋으로 업데이트 가능
- `wgpu` 업데이트는 renderer migration 작업으로 별도 브랜치에서 진행

## Suggested Next Conversation

가장 자연스러운 다음 작업은 아래 중 하나입니다.

1. `RenderWorldExtractor` 테스트와 render boundary 안정화
2. `GPUResourceManager` lookup 오류를 `Result`로 전환
3. dispatcher 명칭을 실제 순차 실행에 맞게 정리
4. rotation physics의 다음 범위 결정

추천 시작점은 **RenderWorldExtractor 테스트 추가와 atlas handle/오류 경계 설계**입니다. 현재 아키텍처를 검증하면서 이후 renderer error handling 작업의 범위를 명확하게 만들 수 있습니다.
