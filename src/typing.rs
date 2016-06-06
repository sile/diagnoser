use erl_type;

struct Node {
    pub inputs: Vec<Input>,
    pub output: Output,
}

pub struct Input {
    pub node: Node,
    pub constraint: Type,
}

pub struct Output {
    pub node: Node,
    pub constraint: Type,
}

pub struct Match {
    pub value: Type,
    pub pattern: Type,
    pub gurad: Type,
}

pub struct Case {
    pub value: Type,
    pub clauses: Vec<Clause>,
}

pub enum Contract {
    }


// [memo]
// 範囲が狭まるケース(intersection):
// - パターンマッチ:
//   - 変数の束縛や関数呼び出しもこの一種
//   - 左辺値と右辺値の型集合の共通部分が、左辺値の新しい型となる
// - ガード:
//   - パターンマッチに対する追加の制約
//
// 範囲が広まるケース(union):
// - clause:
//   - 入出力ともに、全ての節の和集合となる
//
// 環境の変更:
// - 変数束縛全般 (e.g., 新しい変数名が出現したらその都度)

pub struct Node {
    pub use_type: Type,
    pub allow_type: Type,
}

// ---- これより上は古い -----

// [memo]
// 各ノードには"許容する型"と"実際に使われている型"がある:
// - 前者は、ボトムアップに定まるもの (term()から始まる狭まっていく)
// - 後者は、トップダウンに定まるもの (none()から始まり広がっていく)
//   - 内部関数のように、利用者の集合に上限があるものに関しては、前者が後者を包含する場合には、後者を狭める形で利用することが可能
//   - 論文中のrefine周り
//
// 全ての変数(式)は初期値は`{使用:none(), 許容:any()}`となる:
// - "式"といいつつパターンやガードも含む
//   - e.g., ガードの場合には許容型に`true`が含まれている必要がある (そうではないなら、そのガードは無意味)
// - openなuse_typeには、`any()`が必ず付与される
// - allow_typeに関しては、specが指定されているものに関しては、any()ではなく、そのspecの値が初期値となる
// - fixpointに至るまでは、iterationを繰り返す
//   - 有限性の保証は論文と同様
// - use_typeとallow_typeの共通集合がnone()の場合には、型エラー

// [memo:解析の流れ]
// とりあえずは、全てオンメモリで一度に処理すると仮定
//
// 1) 関数呼び出しや型定義を元に、モジュールの依存グラフを構築
// 2) 同時に、ユーザ型の定義の辞書を構築
// 3) 依存グラフの末端モジュールから順に解析していく
//   - 相互依存にあるモジュール群に関しては、escapedな関数のspecが固まるまでiterationされる
//   - => 「変更があったら、その依存元も再解析」というルールに一般化可能
// 4) 一つのモジュール内ではボトムアップとトップダウンの両方(e.g., use_type/allow_type)からの解析を行う
// 5) モジュールを跨ぐ場合には、参照先の関数のallow_typeを利用するのみとする
//   - 利用元のuse_typeを使って、利用先の再解析を行うことはない
// 6) 全てのモジュールのescapedな関数の方がfixpointに到達するまで処理を進める
