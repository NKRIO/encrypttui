pub struct Layer {
    pub ascii: &'static [&'static str],
    // origin 또한 position 처럼 정확한 위치와 비율을 모두 사용할 수 있지만 비율은 실행 환경마다
    // 달라질 수 있는 창 크기가 아닌 하드코딩된 ascii에 대한 것이기 때문에 i16으로 하드코딩한다
    pub origin: (u16, u16),
    pub position: (Position, Position),
}
pub struct Position {
    // 이미지 크기에 따라 유저가 위치를 음의 방향으로 설정할 수 있다.
    pub numerator: i16,
    // 분모는 0이 될 수 없기에 0인 경우 절대 위치를 사용하는 것으로 간주한다.
    // 분자가 이미 signed이므로 분모는 signed일 필요가 없다.
    pub denominator: u16,
    // 이미지 크기에 따라 유저가 위치를 음의 방향으로 설정할 수 있다.
    pub absolute: i16,
    // 거짓일 경우 왼쪽 혹은 위에서 시작하지만 참일 경우 오른쪽 혹은 아래에서 시작함
    pub flip: bool
}
