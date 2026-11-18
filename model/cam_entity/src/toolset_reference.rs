use uuid::Uuid;

/// ツールライブラリの ID（全体で一意で、ファイルを移しても・名前を替えても同じライブラリを指す）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ToolLibraryId(Uuid);

impl ToolLibraryId {
    /// UUID から作る（保存した ID に戻すときに使う）
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

/// ツールセットの ID（全体で一意で、中身を変えても同じツールセットを指す）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ToolSetId(Uuid);

impl ToolSetId {
    /// UUID から作る（保存した ID に戻すときに使う）
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

/// 加工工程の工具を選んだ、ツールライブラリの項目への参照
///
/// 計算には使わない（計算は、加工工程の計算の条件の工具の値で行う）。ライブラリはファイルの場所ではなく ID で引く。
#[derive(Debug, Clone, PartialEq)]
pub struct ToolSetReference {
    /// ツールセットのあるツールライブラリ
    pub library: ToolLibraryId,
    /// ツールセット
    pub toolset: ToolSetId,
    /// 選んだとき（または最後にライブラリと比べたとき）のツールセットの名前（ライブラリが見つからないときに出す）
    pub name: String,
}
