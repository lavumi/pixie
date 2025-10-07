use std::collections::HashMap;
use specs::World;
use winit::event::WindowEvent;
use crate::renderer::{TileRenderData, TextRenderData};

/// Application trait - 게임별 로직을 정의하는 인터페이스
pub trait Application {
    /// 초기화 - 컴포넌트 등록, 리소스 등록, 엔티티 생성
    fn init(&mut self, world: &mut World);

    /// 프레임마다 업데이트 (렌더링/입력 반응 등 비-물리)
    fn update(&mut self, world: &mut World, dt: f32);

    /// 고정 시간 간격 업데이트 (물리 전용). 기본은 no-op
    fn fixed_update(&mut self, _world: &mut World, _fixed_dt: f32) { }

    /// 입력 처리 (반환값: 이벤트 소비 여부)
    fn handle_input(&mut self, world: &mut World, event: &WindowEvent) -> bool;

    /// 카메라 변환 행렬 제공
    fn get_camera_uniform(&self, world: &World) -> [[f32; 4]; 4];

    /// 타일 렌더링 데이터 제공
    fn get_tile_instances(&self, world: &World) -> HashMap<String, Vec<TileRenderData>>;

    /// 텍스트 렌더링 데이터 제공
    fn get_text_instances(&self, world: &World) -> Vec<TextRenderData>;

    /// 고정 스텝을 실행할지 여부 (일시정지/상태에 따라 제어). 기본값: 항상 실행
    fn should_run_fixed(&self, _world: &World) -> bool { true }
}
