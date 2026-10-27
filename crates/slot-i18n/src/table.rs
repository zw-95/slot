use crate::{Lang, MsgKey};

/// The word for `key` in `lang`. Both languages sit in one `match`, so a key that has only one
/// of them does not build.
pub fn text(lang: Lang, key: MsgKey) -> &'static str {
    match (lang, key) {
        // The quick menu's rows.
        (Lang::En, MsgKey::RowFastForward) => "Fast Forward",
        (Lang::ZhCn, MsgKey::RowFastForward) => "快进",

        (Lang::En, MsgKey::RowFastForwardSound) => "Fast Forward Sound",
        (Lang::ZhCn, MsgKey::RowFastForwardSound) => "快进音效",

        (Lang::En, MsgKey::RowScreen) => "Screen",
        (Lang::ZhCn, MsgKey::RowScreen) => "屏幕",

        (Lang::En, MsgKey::RowRumble) => "Rumble",
        (Lang::ZhCn, MsgKey::RowRumble) => "震动",

        (Lang::En, MsgKey::RowDateTime) => "Date & Time",
        (Lang::ZhCn, MsgKey::RowDateTime) => "日期与时间",

        (Lang::En, MsgKey::RowClock) => "Clock",
        (Lang::ZhCn, MsgKey::RowClock) => "时钟",

        (Lang::En, MsgKey::RowMenuSounds) => "Menu Sounds",
        (Lang::ZhCn, MsgKey::RowMenuSounds) => "菜单音效",

        (Lang::En, MsgKey::RowHaptics) => "Haptics",
        (Lang::ZhCn, MsgKey::RowHaptics) => "触感反馈",

        (Lang::En, MsgKey::RowAbout) => "About",
        (Lang::ZhCn, MsgKey::RowAbout) => "关于",

        (Lang::En, MsgKey::RowGameplay) => "Gameplay",
        (Lang::ZhCn, MsgKey::RowGameplay) => "游戏",

        (Lang::En, MsgKey::RowEjectSave) => "Auto Save on Eject",
        (Lang::ZhCn, MsgKey::RowEjectSave) => "退卡时自动存档",

        (Lang::En, MsgKey::RowTurbo) => "Turbo Buttons",
        (Lang::ZhCn, MsgKey::RowTurbo) => "连发键",

        (Lang::En, MsgKey::RowRewind) => "Rewind",
        (Lang::ZhCn, MsgKey::RowRewind) => "倒带",

        (Lang::En, MsgKey::RowPalettes) => "Palettes",
        (Lang::ZhCn, MsgKey::RowPalettes) => "调色板",

        (Lang::En, MsgKey::RowRunAhead) => "Run-Ahead",
        (Lang::ZhCn, MsgKey::RowRunAhead) => "预运行",

        (Lang::En, MsgKey::RowTheme) => "Theme",
        (Lang::ZhCn, MsgKey::RowTheme) => "主题",

        (Lang::En, MsgKey::RowWallpaper) => "Wallpaper",
        (Lang::ZhCn, MsgKey::RowWallpaper) => "壁纸",

        (Lang::En, MsgKey::RowGeneral) => "General",
        (Lang::ZhCn, MsgKey::RowGeneral) => "通用",

        (Lang::En, MsgKey::RowRecents) => "Recents",
        (Lang::ZhCn, MsgKey::RowRecents) => "最近玩过",

        (Lang::En, MsgKey::RowShowTitles) => "Game Titles",
        (Lang::ZhCn, MsgKey::RowShowTitles) => "游戏标题",

        (Lang::En, MsgKey::RowWallpaperTint) => "Wallpaper Tint",
        (Lang::ZhCn, MsgKey::RowWallpaperTint) => "壁纸暗化",

        (Lang::En, MsgKey::RowResume) => "Resume",
        (Lang::ZhCn, MsgKey::RowResume) => "继续游戏",

        (Lang::En, MsgKey::RowStates) => "States",
        (Lang::ZhCn, MsgKey::RowStates) => "存档",

        (Lang::En, MsgKey::RowSettings) => "Settings",
        (Lang::ZhCn, MsgKey::RowSettings) => "设置",

        (Lang::En, MsgKey::RowExitGame) => "Exit Game",
        (Lang::ZhCn, MsgKey::RowExitGame) => "退出游戏",

        (Lang::En, MsgKey::RowShader) => "Shader",
        (Lang::ZhCn, MsgKey::RowShader) => "滤镜",

        (Lang::En, MsgKey::RowColourCorrection) => "Color Correction",
        (Lang::ZhCn, MsgKey::RowColourCorrection) => "色彩校正",

        // "Palette" and "Palettes" are two rows that must not read alike in any tongue: the
        // menu dedupes its rows by wording, and a batch of a different shape would be baked
        // into the textures the last one was baked into. See slot-ui/tests/rows_differ.rs.
        (Lang::En, MsgKey::RowPalette) => "Palette",
        (Lang::ZhCn, MsgKey::RowPalette) => "配色",

        (Lang::En, MsgKey::RowBezel) => "Bezel",
        (Lang::ZhCn, MsgKey::RowBezel) => "边框",

        (Lang::En, MsgKey::RowDisc) => "Disc",
        (Lang::ZhCn, MsgKey::RowDisc) => "碟片",

        (Lang::En, MsgKey::RowNetwork) => "Network",
        (Lang::ZhCn, MsgKey::RowNetwork) => "网络",

        (Lang::En, MsgKey::RowWifi) => "Wi-Fi",
        (Lang::ZhCn, MsgKey::RowWifi) => "Wi-Fi",

        (Lang::En, MsgKey::RowNetworks) => "Networks",
        (Lang::ZhCn, MsgKey::RowNetworks) => "可用网络",

        (Lang::En, MsgKey::RowBluetooth) => "Bluetooth",
        (Lang::ZhCn, MsgKey::RowBluetooth) => "蓝牙",

        (Lang::En, MsgKey::RowDevices) => "Devices",
        (Lang::ZhCn, MsgKey::RowDevices) => "设备",

        (Lang::En, MsgKey::RowServices) => "Services",
        (Lang::ZhCn, MsgKey::RowServices) => "服务",

        (Lang::En, MsgKey::RowUpdates) => "Updates",
        (Lang::ZhCn, MsgKey::RowUpdates) => "更新",

        (Lang::En, MsgKey::RowCores) => "Cores",
        (Lang::ZhCn, MsgKey::RowCores) => "模拟器核心",

        (Lang::En, MsgKey::RowGameArt) => "Game Art",
        (Lang::ZhCn, MsgKey::RowGameArt) => "游戏封面",

        (Lang::En, MsgKey::RowAchievements) => "RetroAchievements",
        (Lang::ZhCn, MsgKey::RowAchievements) => "RetroAchievements",

        (Lang::En, MsgKey::RowPower) => "Power",
        (Lang::ZhCn, MsgKey::RowPower) => "电源",

        (Lang::En, MsgKey::RowRestart) => "Restart",
        (Lang::ZhCn, MsgKey::RowRestart) => "重启",

        (Lang::En, MsgKey::RowShutDown) => "Shut Down",
        (Lang::ZhCn, MsgKey::RowShutDown) => "关机",

        (Lang::En, MsgKey::RowReload) => "Reload unslot",
        (Lang::ZhCn, MsgKey::RowReload) => "重载 unslot",

        (Lang::En, MsgKey::RowLanguage) => "Language",
        (Lang::ZhCn, MsgKey::RowLanguage) => "语言",

        (Lang::En, MsgKey::RowTvSound) => "Sound on TV",
        (Lang::ZhCn, MsgKey::RowTvSound) => "电视声音",

        (Lang::En, MsgKey::RowCast) => "Cast",
        (Lang::ZhCn, MsgKey::RowCast) => "投屏",

        (Lang::En, MsgKey::CastTitle) => "Cast",
        (Lang::ZhCn, MsgKey::CastTitle) => "投屏",

        (Lang::En, MsgKey::CastSearch) => "Search",
        (Lang::ZhCn, MsgKey::CastSearch) => "搜索",

        (Lang::En, MsgKey::CastStop) => "Stop",
        (Lang::ZhCn, MsgKey::CastStop) => "停止",

        (Lang::En, MsgKey::CastCasting) => "Casting",
        (Lang::ZhCn, MsgKey::CastCasting) => "正在投屏",

        (Lang::En, MsgKey::CastLooking) => "Looking for TVs...",
        (Lang::ZhCn, MsgKey::CastLooking) => "正在搜索电视...",

        (Lang::En, MsgKey::CastConnecting) => "Connecting to {}...",
        (Lang::ZhCn, MsgKey::CastConnecting) => "正在连接 {}...",

        (Lang::En, MsgKey::CastCastingTo) => "Casting to {}",
        (Lang::ZhCn, MsgKey::CastCastingTo) => "正在投屏到 {}",

        (Lang::En, MsgKey::CastNoTvs) => "No TVs found. Press Y to look again",
        (Lang::ZhCn, MsgKey::CastNoTvs) => "没有找到电视。按 Y 重新搜索",

        (Lang::En, MsgKey::CastPick) => "Pick a TV to cast the screen to",
        (Lang::ZhCn, MsgKey::CastPick) => "选择一台电视开始投屏",

        // The words a row's setting reads as.
        (Lang::En, MsgKey::ValueOn) => "On",
        (Lang::ZhCn, MsgKey::ValueOn) => "开",

        (Lang::En, MsgKey::ValueOff) => "Off",
        (Lang::ZhCn, MsgKey::ValueOff) => "关",

        (Lang::En, MsgKey::ValueGrid) => "Grid",
        (Lang::ZhCn, MsgKey::ValueGrid) => "网格",

        (Lang::En, MsgKey::ValueDot) => "Dot",
        (Lang::ZhCn, MsgKey::ValueDot) => "点阵",

        (Lang::En, MsgKey::ValueFrames1) => "1 Frame",
        (Lang::ZhCn, MsgKey::ValueFrames1) => "1 帧",

        (Lang::En, MsgKey::ValueDark) => "Dark",
        (Lang::ZhCn, MsgKey::ValueDark) => "深色",

        (Lang::En, MsgKey::ValueLight) => "Light",
        (Lang::ZhCn, MsgKey::ValueLight) => "浅色",

        (Lang::En, MsgKey::ValueShow) => "Show",
        (Lang::ZhCn, MsgKey::ValueShow) => "显示",

        (Lang::En, MsgKey::ValueHide) => "Hide",
        (Lang::ZhCn, MsgKey::ValueHide) => "隐藏",

        (Lang::En, MsgKey::ValueDisc1) => "Disc 1",
        (Lang::ZhCn, MsgKey::ValueDisc1) => "第 1 碟",

        (Lang::En, MsgKey::ValueDisc2) => "Disc 2",
        (Lang::ZhCn, MsgKey::ValueDisc2) => "第 2 碟",

        (Lang::En, MsgKey::ValueDisc3) => "Disc 3",
        (Lang::ZhCn, MsgKey::ValueDisc3) => "第 3 碟",

        (Lang::En, MsgKey::ValueDisc4) => "Disc 4",
        (Lang::ZhCn, MsgKey::ValueDisc4) => "第 4 碟",

        (Lang::En, MsgKey::ValueDisc5) => "Disc 5",
        (Lang::ZhCn, MsgKey::ValueDisc5) => "第 5 碟",

        (Lang::En, MsgKey::ValueHours24) => "24-Hour",
        (Lang::ZhCn, MsgKey::ValueHours24) => "24 小时制",

        (Lang::En, MsgKey::ValueHours12) => "12-Hour",
        (Lang::ZhCn, MsgKey::ValueHours12) => "12 小时制",

        // A language is named in its own tongue, whatever the interface is in.
        (Lang::En, MsgKey::ValueLangEn) | (Lang::ZhCn, MsgKey::ValueLangEn) => "English",
        (Lang::En, MsgKey::ValueLangZh) | (Lang::ZhCn, MsgKey::ValueLangZh) => "简体中文",

        // The banners that come up over the game.
        (Lang::En, MsgKey::ToastStateSaved) => "State Saved",
        (Lang::ZhCn, MsgKey::ToastStateSaved) => "存档已保存",

        (Lang::En, MsgKey::ToastStateLoaded) => "State Loaded",
        (Lang::ZhCn, MsgKey::ToastStateLoaded) => "存档已读取",

        (Lang::En, MsgKey::ToastNeedsGpsp) => "Please switch to gpSP",
        (Lang::ZhCn, MsgKey::ToastNeedsGpsp) => "请切换到 gpSP",

        (Lang::En, MsgKey::ToastNoLink) => "No link support",
        (Lang::ZhCn, MsgKey::ToastNoLink) => "不支持联机",

        (Lang::En, MsgKey::ToastLinkEnded) => "Link ended",
        (Lang::ZhCn, MsgKey::ToastLinkEnded) => "联机已结束",

        (Lang::En, MsgKey::ToastPeerEnded) => "Link was ended",
        (Lang::ZhCn, MsgKey::ToastPeerEnded) => "对方结束了联机",

        (Lang::En, MsgKey::ToastBiosMismatch) => "BIOS does not match",
        (Lang::ZhCn, MsgKey::ToastBiosMismatch) => "BIOS 不一致",

        (Lang::En, MsgKey::ToastColourOn) => "Correction On",
        (Lang::ZhCn, MsgKey::ToastColourOn) => "色彩校正：开",

        (Lang::En, MsgKey::ToastColourOff) => "Correction Off",
        (Lang::ZhCn, MsgKey::ToastColourOff) => "色彩校正：关",

        (Lang::En, MsgKey::ToastShaderOff) => "Shader: Off",
        (Lang::ZhCn, MsgKey::ToastShaderOff) => "滤镜：关",

        (Lang::En, MsgKey::ToastShaderLcd3x) => "Shader: LCD3x",
        (Lang::ZhCn, MsgKey::ToastShaderLcd3x) => "滤镜：LCD3x",

        (Lang::En, MsgKey::ToastShaderGrid) => "Shader: Grid",
        (Lang::ZhCn, MsgKey::ToastShaderGrid) => "滤镜：网格",

        (Lang::En, MsgKey::ToastShaderDot) => "Shader: Dot",
        (Lang::ZhCn, MsgKey::ToastShaderDot) => "滤镜：点阵",

        (Lang::En, MsgKey::ToastShaderSimpletex) => "Shader: Simpletex",
        (Lang::ZhCn, MsgKey::ToastShaderSimpletex) => "滤镜：Simpletex",

        (Lang::En, MsgKey::ToastCardUnwritable) => "Can't write to SD card",
        (Lang::ZhCn, MsgKey::ToastCardUnwritable) => "SD 卡无法写入",

        (Lang::En, MsgKey::ToastNoBios) => "BIOS missing",
        (Lang::ZhCn, MsgKey::ToastNoBios) => "缺少 BIOS",

        (Lang::En, MsgKey::ToastCardDetail) => "Progress is not being saved",
        (Lang::ZhCn, MsgKey::ToastCardDetail) => "进度不会被保存",

        // The keyboard's own title.
        (Lang::En, MsgKey::KbSearchForAGame) => "Search for a game",
        (Lang::ZhCn, MsgKey::KbSearchForAGame) => "搜索游戏",

        // The words under the buttons.
        (Lang::En, MsgKey::BtnBack) => "Back",
        (Lang::ZhCn, MsgKey::BtnBack) => "返回",

        (Lang::En, MsgKey::BtnChange) => "Change",
        (Lang::ZhCn, MsgKey::BtnChange) => "更改",

        (Lang::En, MsgKey::BtnOpen) => "Open",
        (Lang::ZhCn, MsgKey::BtnOpen) => "打开",

        (Lang::En, MsgKey::BtnCancel) => "Cancel",
        (Lang::ZhCn, MsgKey::BtnCancel) => "取消",

        (Lang::En, MsgKey::BtnSwap) => "Swap",
        (Lang::ZhCn, MsgKey::BtnSwap) => "切换",

        (Lang::En, MsgKey::BtnChoose) => "Choose",
        (Lang::ZhCn, MsgKey::BtnChoose) => "选择",

        (Lang::En, MsgKey::BtnAllGba) => "All GBA",
        (Lang::ZhCn, MsgKey::BtnAllGba) => "全部 GBA",

        (Lang::En, MsgKey::BtnMode) => "Mode",
        (Lang::ZhCn, MsgKey::BtnMode) => "模式",

        (Lang::En, MsgKey::BtnLink) => "Link",
        (Lang::ZhCn, MsgKey::BtnLink) => "联机",

        (Lang::En, MsgKey::BtnOk) => "OK",
        (Lang::ZhCn, MsgKey::BtnOk) => "确定",

        (Lang::En, MsgKey::BtnEndLink) => "End Link",
        (Lang::ZhCn, MsgKey::BtnEndLink) => "结束联机",

        (Lang::En, MsgKey::BtnDelete) => "Delete",
        (Lang::ZhCn, MsgKey::BtnDelete) => "删除",

        (Lang::En, MsgKey::BtnLoad) => "Load",
        (Lang::ZhCn, MsgKey::BtnLoad) => "读取",

        (Lang::En, MsgKey::BtnSpace) => "Space",
        (Lang::ZhCn, MsgKey::BtnSpace) => "空格",

        (Lang::En, MsgKey::BtnShift) => "Shift",
        (Lang::ZhCn, MsgKey::BtnShift) => "上档",

        (Lang::En, MsgKey::BtnSymbols) => "Symbols",
        (Lang::ZhCn, MsgKey::BtnSymbols) => "符号",

        (Lang::En, MsgKey::BtnDone) => "Done",
        (Lang::ZhCn, MsgKey::BtnDone) => "完成",

        // The keyboard's caps key.
        (Lang::En, MsgKey::KbCaps) => "CAPS",
        (Lang::ZhCn, MsgKey::KbCaps) => "大写",

        // Where the machine is going, now that it has been told to stop.
        (Lang::En, MsgKey::PowerRestarting) => "Restarting",
        (Lang::ZhCn, MsgKey::PowerRestarting) => "正在重启",

        (Lang::En, MsgKey::PoweringDown) => "Powering Down",
        (Lang::ZhCn, MsgKey::PoweringDown) => "正在关机",

        (Lang::En, MsgKey::PowerReloading) => "Reloading",
        (Lang::ZhCn, MsgKey::PowerReloading) => "正在重载",

        // Linking two machines together.
        (Lang::En, MsgKey::LinkHost) => "Host",
        (Lang::ZhCn, MsgKey::LinkHost) => "主机",

        (Lang::En, MsgKey::LinkJoin) => "Join",
        (Lang::ZhCn, MsgKey::LinkJoin) => "加入",

        (Lang::En, MsgKey::LinkLinked) => "Linked",
        (Lang::ZhCn, MsgKey::LinkLinked) => "已连接",

        (Lang::En, MsgKey::LinkStepRadio) => "Bringing the radio up",
        (Lang::ZhCn, MsgKey::LinkStepRadio) => "正在开启无线",

        (Lang::En, MsgKey::LinkStepWaiting) => "Looking for the other player",
        (Lang::ZhCn, MsgKey::LinkStepWaiting) => "正在寻找另一台设备",

        (Lang::En, MsgKey::LinkFailRadio) => "The radio did not come up",
        (Lang::ZhCn, MsgKey::LinkFailRadio) => "无线没能开启",

        (Lang::En, MsgKey::LinkFailNobody) => "Nobody arrived",
        (Lang::ZhCn, MsgKey::LinkFailNobody) => "没有设备加入",

        (Lang::En, MsgKey::LinkFailVanished) => "The other player vanished",
        (Lang::ZhCn, MsgKey::LinkFailVanished) => "对方断开了连接",

        (Lang::En, MsgKey::LinkFailCancelled) => "Cancelled",
        (Lang::ZhCn, MsgKey::LinkFailCancelled) => "已取消",

        // The clock.
        (Lang::En, MsgKey::ClockSetHint) => "set the clock",
        (Lang::ZhCn, MsgKey::ClockSetHint) => "设置时间",

        (Lang::En, MsgKey::TimeAm) => "AM",
        (Lang::ZhCn, MsgKey::TimeAm) => "上午",

        (Lang::En, MsgKey::TimePm) => "PM",
        (Lang::ZhCn, MsgKey::TimePm) => "下午",

        (Lang::En, MsgKey::TimeJustNow) => "just now",
        (Lang::ZhCn, MsgKey::TimeJustNow) => "刚刚",

        (Lang::En, MsgKey::TimeMinAgo) => "{} min ago",
        (Lang::ZhCn, MsgKey::TimeMinAgo) => "{} 分钟前",

        (Lang::En, MsgKey::TimeHrAgo) => "{} hr ago",
        (Lang::ZhCn, MsgKey::TimeHrAgo) => "{} 小时前",

        // What the radio says when it will not do what it was asked.
        (Lang::En, MsgKey::WifiWrongPassword) => "Wrong password",
        (Lang::ZhCn, MsgKey::WifiWrongPassword) => "密码错误",

        (Lang::En, MsgKey::WifiNotInRange) => "Not in range",
        (Lang::ZhCn, MsgKey::WifiNotInRange) => "不在信号范围内",

        (Lang::En, MsgKey::WifiCouldNotConnect) => "Could not connect",
        (Lang::ZhCn, MsgKey::WifiCouldNotConnect) => "连接失败",

        (Lang::En, MsgKey::WifiNoAnswer) => "The Wi-Fi did not answer: {}",
        (Lang::ZhCn, MsgKey::WifiNoAnswer) => "Wi-Fi 没有响应：{}",

        (Lang::En, MsgKey::WifiWouldNotStart) => "The Wi-Fi would not start",
        (Lang::ZhCn, MsgKey::WifiWouldNotStart) => "Wi-Fi 无法启动",

        (Lang::En, MsgKey::WifiWouldNotTake) => "The Wi-Fi would not take a network",
        (Lang::ZhCn, MsgKey::WifiWouldNotTake) => "Wi-Fi 不接受这个网络",

        (Lang::En, MsgKey::BtWouldNotStart) => "Bluetooth would not start",
        (Lang::ZhCn, MsgKey::BtWouldNotStart) => "蓝牙无法启动",

        (Lang::En, MsgKey::BtCouldNotConnect) => "Could not connect to {}",
        (Lang::ZhCn, MsgKey::BtCouldNotConnect) => "无法连接到 {}",

        (Lang::En, MsgKey::BtWouldNotPair) => {
            "{} would not pair. Turn Bluetooth off and on, then try again"
        }
        (Lang::ZhCn, MsgKey::BtWouldNotPair) => "{} 配对失败。请关闭蓝牙再打开，然后重试",

        (Lang::En, MsgKey::MonthJan) => "JAN",
        (Lang::ZhCn, MsgKey::MonthJan) => "1 月",

        (Lang::En, MsgKey::MonthFeb) => "FEB",
        (Lang::ZhCn, MsgKey::MonthFeb) => "2 月",

        (Lang::En, MsgKey::MonthMar) => "MAR",
        (Lang::ZhCn, MsgKey::MonthMar) => "3 月",

        (Lang::En, MsgKey::MonthApr) => "APR",
        (Lang::ZhCn, MsgKey::MonthApr) => "4 月",

        (Lang::En, MsgKey::MonthMay) => "MAY",
        (Lang::ZhCn, MsgKey::MonthMay) => "5 月",

        (Lang::En, MsgKey::MonthJun) => "JUN",
        (Lang::ZhCn, MsgKey::MonthJun) => "6 月",

        (Lang::En, MsgKey::MonthJul) => "JUL",
        (Lang::ZhCn, MsgKey::MonthJul) => "7 月",

        (Lang::En, MsgKey::MonthAug) => "AUG",
        (Lang::ZhCn, MsgKey::MonthAug) => "8 月",

        (Lang::En, MsgKey::MonthSep) => "SEP",
        (Lang::ZhCn, MsgKey::MonthSep) => "9 月",

        (Lang::En, MsgKey::MonthOct) => "OCT",
        (Lang::ZhCn, MsgKey::MonthOct) => "10 月",

        (Lang::En, MsgKey::MonthNov) => "NOV",
        (Lang::ZhCn, MsgKey::MonthNov) => "11 月",

        (Lang::En, MsgKey::MonthDec) => "DEC",
        (Lang::ZhCn, MsgKey::MonthDec) => "12 月",

        // Offering to take back what was just done to a save.
        (Lang::En, MsgKey::UndoSave) => "undo save",
        (Lang::ZhCn, MsgKey::UndoSave) => "撤销存档",

        (Lang::En, MsgKey::UndoLoad) => "undo load",
        (Lang::ZhCn, MsgKey::UndoLoad) => "撤销读档",

        // The rebuild screen's one line of type.
        (Lang::En, MsgKey::RebuildCaching) => "Caching New Labels",
        (Lang::ZhCn, MsgKey::RebuildCaching) => "正在缓存新封面",

        // The two racks that are not a system.
        (Lang::En, MsgKey::RackRecents) => "Recents",
        (Lang::ZhCn, MsgKey::RackRecents) => "最近玩过",

        (Lang::En, MsgKey::RackFavorites) => "Favorites",
        (Lang::ZhCn, MsgKey::RackFavorites) => "收藏",

        // Choosing a core, and the page that does it.
        (Lang::En, MsgKey::PageCore) => "Core",
        (Lang::ZhCn, MsgKey::PageCore) => "核心",

        (Lang::En, MsgKey::PageCoreOptions) => "Core Options",
        (Lang::ZhCn, MsgKey::PageCoreOptions) => "核心选项",

        (Lang::En, MsgKey::CoreInUse) => "In use",
        (Lang::ZhCn, MsgKey::CoreInUse) => "使用中",

        (Lang::En, MsgKey::CoreDefault) => "Default",
        (Lang::ZhCn, MsgKey::CoreDefault) => "默认",

        // Choosing a shell, and the page that does it.
        (Lang::En, MsgKey::PageShell) => "Shell",
        (Lang::ZhCn, MsgKey::PageShell) => "外壳",

        (Lang::En, MsgKey::ShellTemplate) => "Template {}",
        (Lang::ZhCn, MsgKey::ShellTemplate) => "模版 {}",

        (Lang::En, MsgKey::ShellInUse) => "In use",
        (Lang::ZhCn, MsgKey::ShellInUse) => "使用中",

        (Lang::En, MsgKey::PlasticGrey) => "Grey",
        (Lang::ZhCn, MsgKey::PlasticGrey) => "灰",

        (Lang::En, MsgKey::PlasticBlack) => "Black",
        (Lang::ZhCn, MsgKey::PlasticBlack) => "黑",

        (Lang::En, MsgKey::PlasticWhite) => "White",
        (Lang::ZhCn, MsgKey::PlasticWhite) => "白",

        (Lang::En, MsgKey::PlasticBlue) => "Blue",
        (Lang::ZhCn, MsgKey::PlasticBlue) => "蓝",

        (Lang::En, MsgKey::PlasticPink) => "Pink",
        (Lang::ZhCn, MsgKey::PlasticPink) => "粉",

        (Lang::En, MsgKey::PlasticRed) => "Red",
        (Lang::ZhCn, MsgKey::PlasticRed) => "红",

        (Lang::En, MsgKey::PlasticGreen) => "Green",
        (Lang::ZhCn, MsgKey::PlasticGreen) => "绿",

        (Lang::En, MsgKey::PlasticYellow) => "Yellow",
        (Lang::ZhCn, MsgKey::PlasticYellow) => "黄",

        (Lang::En, MsgKey::CoreThisGame) => "This Game",
        (Lang::ZhCn, MsgKey::CoreThisGame) => "仅本游戏",

        (Lang::En, MsgKey::CoreAllOf) => "All {}",
        (Lang::ZhCn, MsgKey::CoreAllOf) => "全部 {}",

        (Lang::En, MsgKey::ArtWaiting) => "Waiting",
        (Lang::ZhCn, MsgKey::ArtWaiting) => "等待中",

        (Lang::En, MsgKey::ArtDownloaded) => "Downloaded",
        (Lang::ZhCn, MsgKey::ArtDownloaded) => "已下载",

        (Lang::En, MsgKey::SearchNew) => "New Search",
        (Lang::ZhCn, MsgKey::SearchNew) => "重新搜索",

        (Lang::En, MsgKey::SearchGoToGame) => "Go to Game",
        (Lang::ZhCn, MsgKey::SearchGoToGame) => "前往游戏",

        (Lang::En, MsgKey::PageSearch) => "Search",
        (Lang::ZhCn, MsgKey::PageSearch) => "搜索",

        (Lang::En, MsgKey::SearchNoMatch) => "No games match \u{201c}{}\u{201d}",
        (Lang::ZhCn, MsgKey::SearchNoMatch) => "没有匹配“{}”的游戏",

        (Lang::En, MsgKey::SearchOneMatch) => "1 game matches \u{201c}{}\u{201d}",
        (Lang::ZhCn, MsgKey::SearchOneMatch) => "有 1 个游戏匹配“{}”",

        (Lang::En, MsgKey::SearchFirstFew) => "The first {n} games matching \u{201c}{}\u{201d}",
        (Lang::ZhCn, MsgKey::SearchFirstFew) => "与“{}”匹配的前 {n} 个游戏",

        (Lang::En, MsgKey::SearchManyMatch) => "{n} games match \u{201c}{}\u{201d}",
        (Lang::ZhCn, MsgKey::SearchManyMatch) => "有 {n} 个游戏匹配“{}”",

        // The services page.
        (Lang::En, MsgKey::PageServices) => "Services",
        (Lang::ZhCn, MsgKey::PageServices) => "服务",

        (Lang::En, MsgKey::ServiceSsh) => "SSH",
        (Lang::ZhCn, MsgKey::ServiceSsh) => "SSH",

        (Lang::En, MsgKey::ServiceSharing) => "File sharing",
        (Lang::ZhCn, MsgKey::ServiceSharing) => "文件共享",

        (Lang::En, MsgKey::ServiceSyncthing) => "Syncthing",
        (Lang::ZhCn, MsgKey::ServiceSyncthing) => "Syncthing",

        (Lang::En, MsgKey::ServiceTailscale) => "Tailscale",
        (Lang::ZhCn, MsgKey::ServiceTailscale) => "Tailscale",

        (Lang::En, MsgKey::ServiceVpn) => "VPN",
        (Lang::ZhCn, MsgKey::ServiceVpn) => "VPN",

        (Lang::En, MsgKey::ServicePassword) => "Password",
        (Lang::ZhCn, MsgKey::ServicePassword) => "密码",

        (Lang::En, MsgKey::ServicePasswordBaseOs) => "BaseOS's",
        (Lang::ZhCn, MsgKey::ServicePasswordBaseOs) => "跟随 BaseOS",

        (Lang::En, MsgKey::DetailStarting) => "Starting",
        (Lang::ZhCn, MsgKey::DetailStarting) => "启动中",

        (Lang::En, MsgKey::DetailSignIn) => "Sign in",
        (Lang::ZhCn, MsgKey::DetailSignIn) => "登录",

        (Lang::En, MsgKey::DetailNeedsApproval) => "Needs approval",
        (Lang::ZhCn, MsgKey::DetailNeedsApproval) => "需要授权",

        (Lang::En, MsgKey::DetailConnecting) => "Connecting",
        (Lang::ZhCn, MsgKey::DetailConnecting) => "连接中",

        (Lang::En, MsgKey::DetailNotConnected) => "Not connected",
        (Lang::ZhCn, MsgKey::DetailNotConnected) => "未连接",

        (Lang::En, MsgKey::DetailSet) => "Set",
        (Lang::ZhCn, MsgKey::DetailSet) => "已设置",

        (Lang::En, MsgKey::BtnTurnOn) => "Turn On",
        (Lang::ZhCn, MsgKey::BtnTurnOn) => "开启",

        (Lang::En, MsgKey::BtnTurnOff) => "Turn Off",
        (Lang::ZhCn, MsgKey::BtnTurnOff) => "关闭",

        (Lang::En, MsgKey::BtnChangeTunnel) => "Change Tunnel",
        (Lang::ZhCn, MsgKey::BtnChangeTunnel) => "更换隧道",

        (Lang::En, MsgKey::BtnChangePassword) => "Change Password",
        (Lang::ZhCn, MsgKey::BtnChangePassword) => "更改密码",

        (Lang::En, MsgKey::BtnSetPassword) => "Set Password",
        (Lang::ZhCn, MsgKey::BtnSetPassword) => "设置密码",

        (Lang::En, MsgKey::BtnShowId) => "Show ID",
        (Lang::ZhCn, MsgKey::BtnShowId) => "显示 ID",

        (Lang::En, MsgKey::NetNeedWifi) => "Connect to Wi-Fi to reach these",
        (Lang::ZhCn, MsgKey::NetNeedWifi) => "连接 Wi-Fi 后才能使用",

        (Lang::En, MsgKey::VpnConfigHint) => "Put WireGuard configs in {} on the card",
        (Lang::ZhCn, MsgKey::VpnConfigHint) => "把 WireGuard 配置放进卡上的 {}",

        (Lang::En, MsgKey::NewPasswordForRoot) => "New password for root",
        (Lang::ZhCn, MsgKey::NewPasswordForRoot) => "root 的新密码",

        (Lang::En, MsgKey::LoginSaveFail) => "Could not save the login: {}",
        (Lang::ZhCn, MsgKey::LoginSaveFail) => "登录信息保存失败：{}",

        // The game art page.
        (Lang::En, MsgKey::PageGameArt) => "Game Art",
        (Lang::ZhCn, MsgKey::PageGameArt) => "游戏封面",

        (Lang::En, MsgKey::ArtAllSystems) => "All systems",
        (Lang::ZhCn, MsgKey::ArtAllSystems) => "所有机种",

        (Lang::En, MsgKey::ArtEveryGameHasLabel) => "Every game here has a label",
        (Lang::ZhCn, MsgKey::ArtEveryGameHasLabel) => "这里每个游戏都有封面",

        (Lang::En, MsgKey::ArtHasCount) => "{} of {}",
        (Lang::ZhCn, MsgKey::ArtHasCount) => "{} / {}",

        (Lang::En, MsgKey::ArtFinding) => "Finding art for {} ({} of {})",
        (Lang::ZhCn, MsgKey::ArtFinding) => "正在为 {} 找封面（{} / {}）",

        (Lang::En, MsgKey::ArtSkipped) => "Skipped {} game{} ScreenScraper turned away",
        (Lang::ZhCn, MsgKey::ArtSkipped) => "跳过了 {} 个游戏：ScreenScraper 拒绝了",

        (Lang::En, MsgKey::ArtAdded) => "Added {} label{} to the shelves",
        (Lang::ZhCn, MsgKey::ArtAdded) => "已给书架加上 {} 张封面",

        (Lang::En, MsgKey::ArtNoneFound) => "No art found for {} game{}",
        (Lang::ZhCn, MsgKey::ArtNoneFound) => "有 {} 个游戏没找到封面",

        (Lang::En, MsgKey::ArtFromBoth) => "Art from ScreenScraper and libretro",
        (Lang::ZhCn, MsgKey::ArtFromBoth) => "封面来自 ScreenScraper 与 libretro",

        (Lang::En, MsgKey::ArtNeedsDevId) => "Signed in, but ScreenScraper needs unslot's developer ID",
        (Lang::ZhCn, MsgKey::ArtNeedsDevId) => "已登录，但 ScreenScraper 需要 unslot 的开发者 ID",

        (Lang::En, MsgKey::ArtFromLibretro) => "Art from libretro-thumbnails",
        (Lang::ZhCn, MsgKey::ArtFromLibretro) => "封面来自 libretro-thumbnails",

        (Lang::En, MsgKey::BtnChangeAccount) => "Change Account",
        (Lang::ZhCn, MsgKey::BtnChangeAccount) => "更换账号",

        (Lang::En, MsgKey::BtnFindMissingArt) => "Find Missing Art",
        (Lang::ZhCn, MsgKey::BtnFindMissingArt) => "补齐缺失封面",

        (Lang::En, MsgKey::ScreenScraperUserPrompt) => "ScreenScraper username, or none to sign out",
        (Lang::ZhCn, MsgKey::ScreenScraperUserPrompt) => "ScreenScraper 用户名，留空则退出登录",

        (Lang::En, MsgKey::ScreenScraperPasswordPrompt) => "ScreenScraper password for {}",
        (Lang::ZhCn, MsgKey::ScreenScraperPasswordPrompt) => "{} 的 ScreenScraper 密码",

        // The updates page.
        (Lang::En, MsgKey::PageUpdates) => "Updates",
        (Lang::ZhCn, MsgKey::PageUpdates) => "更新",

        (Lang::En, MsgKey::UpdateUnknown) => "Unknown",
        (Lang::ZhCn, MsgKey::UpdateUnknown) => "未知",

        (Lang::En, MsgKey::UpdateChecking) => "Checking",
        (Lang::ZhCn, MsgKey::UpdateChecking) => "检查中",

        (Lang::En, MsgKey::UpdateUpToDate) => "Up to date",
        (Lang::ZhCn, MsgKey::UpdateUpToDate) => "已是最新",

        (Lang::En, MsgKey::UpdateAvailable) => "{} available",
        (Lang::ZhCn, MsgKey::UpdateAvailable) => "{} 可用",

        (Lang::En, MsgKey::UpdateReady) => "Ready to install",
        (Lang::ZhCn, MsgKey::UpdateReady) => "可以安装",

        (Lang::En, MsgKey::UpdateDownloading) => "Downloading {} {}...",
        (Lang::ZhCn, MsgKey::UpdateDownloading) => "正在下载 {} {}...",

        (Lang::En, MsgKey::UpdateKeepPower) => "Installs when the device restarts. Keep the power connected",
        (Lang::ZhCn, MsgKey::UpdateKeepPower) => "重启后安装，请保持供电",

        (Lang::En, MsgKey::UpdateChecking2) => "Checking for updates...",
        (Lang::ZhCn, MsgKey::UpdateChecking2) => "正在检查更新...",

        (Lang::En, MsgKey::UpdateOwnBuild) => "(own build)",
        (Lang::ZhCn, MsgKey::UpdateOwnBuild) => "（自编译版本）",

        (Lang::En, MsgKey::BtnCheckAgain) => "Check Again",
        (Lang::ZhCn, MsgKey::BtnCheckAgain) => "重新检查",

        (Lang::En, MsgKey::BtnDownloadVersion) => "Download {}",
        (Lang::ZhCn, MsgKey::BtnDownloadVersion) => "下载 {}",

        (Lang::En, MsgKey::BtnRestartToInstall) => "Restart to Install",
        (Lang::ZhCn, MsgKey::BtnRestartToInstall) => "重启并安装",

        (Lang::En, MsgKey::BtnCheckForUpdates) => "Check for Updates",
        (Lang::ZhCn, MsgKey::BtnCheckForUpdates) => "检查更新",

        // The cores page.
        (Lang::En, MsgKey::PageCores) => "Cores",
        (Lang::ZhCn, MsgKey::PageCores) => "模拟器核心",

        (Lang::En, MsgKey::CoreDownloading) => "Downloading {}...",
        (Lang::ZhCn, MsgKey::CoreDownloading) => "正在下载 {}...",

        (Lang::En, MsgKey::CoreChecking) => "Checking for cores...",
        (Lang::ZhCn, MsgKey::CoreChecking) => "正在检查核心...",

        (Lang::En, MsgKey::CoreHaveCount) => "{} of {} downloaded",
        (Lang::ZhCn, MsgKey::CoreHaveCount) => "已下载 {} / {}",

        (Lang::En, MsgKey::BtnRemove) => "Remove",
        (Lang::ZhCn, MsgKey::BtnRemove) => "移除",

        (Lang::En, MsgKey::BtnDownload) => "Download",
        (Lang::ZhCn, MsgKey::BtnDownload) => "下载",

        (Lang::En, MsgKey::BtnUpdate) => "Update",
        (Lang::ZhCn, MsgKey::BtnUpdate) => "更新",

        // The bluetooth page.
        (Lang::En, MsgKey::PageBluetooth) => "Bluetooth",
        (Lang::ZhCn, MsgKey::PageBluetooth) => "蓝牙",

        (Lang::En, MsgKey::BtConnected) => "Connected",
        (Lang::ZhCn, MsgKey::BtConnected) => "已连接",

        (Lang::En, MsgKey::BtPaired) => "Paired",
        (Lang::ZhCn, MsgKey::BtPaired) => "已配对",

        (Lang::En, MsgKey::BtController) => "Controller",
        (Lang::ZhCn, MsgKey::BtController) => "手柄",

        (Lang::En, MsgKey::BtHeadphones) => "Headphones",
        (Lang::ZhCn, MsgKey::BtHeadphones) => "耳机",

        (Lang::En, MsgKey::BtTurningOn) => "Turning on Bluetooth...",
        (Lang::ZhCn, MsgKey::BtTurningOn) => "正在开启蓝牙...",

        (Lang::En, MsgKey::BtSearching) => "Searching for devices...",
        (Lang::ZhCn, MsgKey::BtSearching) => "正在搜索设备...",

        (Lang::En, MsgKey::BtPairing) => "Pairing {}...",
        (Lang::ZhCn, MsgKey::BtPairing) => "正在与 {} 配对...",

        (Lang::En, MsgKey::BtConnectingTo) => "Connecting {}...",
        (Lang::ZhCn, MsgKey::BtConnectingTo) => "正在连接 {}...",

        (Lang::En, MsgKey::BtPairingHint) => "Put a device in pairing mode, then press Y to search",
        (Lang::ZhCn, MsgKey::BtPairingHint) => "让设备进入配对模式，然后按 Y 搜索",

        (Lang::En, MsgKey::BtNothingConnected) => "Nothing connected",
        (Lang::ZhCn, MsgKey::BtNothingConnected) => "没有连接任何设备",

        (Lang::En, MsgKey::BtOneConnected) => "1 device connected",
        (Lang::ZhCn, MsgKey::BtOneConnected) => "已连接 1 个设备",

        (Lang::En, MsgKey::BtManyConnected) => "{} devices connected",
        (Lang::ZhCn, MsgKey::BtManyConnected) => "已连接 {} 个设备",

        (Lang::En, MsgKey::BtnSearch) => "Search",
        (Lang::ZhCn, MsgKey::BtnSearch) => "搜索",

        (Lang::En, MsgKey::BtnForget) => "Forget",
        (Lang::ZhCn, MsgKey::BtnForget) => "忘记设备",

        (Lang::En, MsgKey::BtnDisconnect) => "Disconnect",
        (Lang::ZhCn, MsgKey::BtnDisconnect) => "断开",

        (Lang::En, MsgKey::BtnConnect) => "Connect",
        (Lang::ZhCn, MsgKey::BtnConnect) => "连接",

        (Lang::En, MsgKey::BtnPair) => "Pair",
        (Lang::ZhCn, MsgKey::BtnPair) => "配对",

        // The Wi-Fi page.
        (Lang::En, MsgKey::PageWifi) => "Wi-Fi",
        (Lang::ZhCn, MsgKey::PageWifi) => "Wi-Fi",

        (Lang::En, MsgKey::WifiNetworkName) => "Network name",
        (Lang::ZhCn, MsgKey::WifiNetworkName) => "网络名称",

        (Lang::En, MsgKey::WifiAlreadyOn) => "Already connected to {}",
        (Lang::ZhCn, MsgKey::WifiAlreadyOn) => "已经连接到 {}",

        (Lang::En, MsgKey::WifiEnterpriseUnsupported) => "Enterprise and WEP networks are not supported",
        (Lang::ZhCn, MsgKey::WifiEnterpriseUnsupported) => "不支持企业网络与 WEP",

        (Lang::En, MsgKey::WifiPasswordPrompt) => "Password for {}",
        (Lang::ZhCn, MsgKey::WifiPasswordPrompt) => "{} 的密码",

        (Lang::En, MsgKey::WifiTurningOn) => "Turning on Wi-Fi...",
        (Lang::ZhCn, MsgKey::WifiTurningOn) => "正在开启 Wi-Fi...",

        (Lang::En, MsgKey::WifiJoining) => "Joining {}...",
        (Lang::ZhCn, MsgKey::WifiJoining) => "正在加入 {}...",

        (Lang::En, MsgKey::WifiSearching) => "Searching for networks...",
        (Lang::ZhCn, MsgKey::WifiSearching) => "正在搜索网络...",

        (Lang::En, MsgKey::WifiConnectedWithIp) => "Connected to {}, {}",
        (Lang::ZhCn, MsgKey::WifiConnectedWithIp) => "已连接到 {}，{}",

        (Lang::En, MsgKey::WifiConnected) => "Connected to {}",
        (Lang::ZhCn, MsgKey::WifiConnected) => "已连接到 {}",

        (Lang::En, MsgKey::BtnJoin) => "Join",
        (Lang::ZhCn, MsgKey::BtnJoin) => "加入",

        (Lang::En, MsgKey::BtnJoinOther) => "Join Other",
        (Lang::ZhCn, MsgKey::BtnJoinOther) => "加入其他",

        (Lang::En, MsgKey::BtnOtherNetwork) => "Other network...",
        (Lang::ZhCn, MsgKey::BtnOtherNetwork) => "其他网络...",

        (Lang::En, MsgKey::WifiSaved) => "Saved",
        (Lang::ZhCn, MsgKey::WifiSaved) => "已保存",

        (Lang::En, MsgKey::WifiNotSupported) => "Not supported",
        (Lang::ZhCn, MsgKey::WifiNotSupported) => "不支持",

        (Lang::En, MsgKey::WifiOpen) => "Open",
        (Lang::ZhCn, MsgKey::WifiOpen) => "开放网络",

        // RetroAchievements, which keeps its own name in either tongue.
        (Lang::En, MsgKey::RaUserPrompt) => "RetroAchievements username, or none to sign out",
        (Lang::ZhCn, MsgKey::RaUserPrompt) => "RetroAchievements 用户名，留空则退出登录",

        (Lang::En, MsgKey::RaPasswordPrompt) => "RetroAchievements password for {}",
        (Lang::ZhCn, MsgKey::RaPasswordPrompt) => "{} 的 RetroAchievements 密码",

        (Lang::En, MsgKey::RaPasswordPromptOpen) => "Password for {}, or none if it is open",
        (Lang::ZhCn, MsgKey::RaPasswordPromptOpen) => "{} 的密码，如果没有密码请留空",

        (Lang::En, MsgKey::RaSigningIn) => "Signing in to RetroAchievements...",
        (Lang::ZhCn, MsgKey::RaSigningIn) => "正在登录 RetroAchievements...",

        (Lang::En, MsgKey::RaSignedOut) => "Signed out of RetroAchievements",
        (Lang::ZhCn, MsgKey::RaSignedOut) => "已退出 RetroAchievements",

        (Lang::En, MsgKey::RaSignOutFail) => "Could not sign out: {}",
        (Lang::ZhCn, MsgKey::RaSignOutFail) => "退出失败：{}",

        (Lang::En, MsgKey::RaNoCardSignOut) => "No card to sign out on",
        (Lang::ZhCn, MsgKey::RaNoCardSignOut) => "没有卡可以保存退出状态",

        (Lang::En, MsgKey::RaSignInFail) => "RetroAchievements did not sign you in",
        (Lang::ZhCn, MsgKey::RaSignInFail) => "RetroAchievements 登录失败",

        (Lang::En, MsgKey::RaSignedInAs) => "Signed in to RetroAchievements as {}",
        (Lang::ZhCn, MsgKey::RaSignedInAs) => "已以 {} 的身份登录 RetroAchievements",

        (Lang::En, MsgKey::RaSignInSaveFail) => "Could not save the sign in: {}",
        (Lang::ZhCn, MsgKey::RaSignInSaveFail) => "登录信息保存失败：{}",

        (Lang::En, MsgKey::RaNoCardSignIn) => "No card to keep the sign in on",
        (Lang::ZhCn, MsgKey::RaNoCardSignIn) => "没有卡可以保存登录信息",

        (Lang::En, MsgKey::RaScanToSignIn) => "Scan to sign in, or open the link on another device",
        (Lang::ZhCn, MsgKey::RaScanToSignIn) => "扫码登录，或在另一台设备上打开链接",

        (Lang::En, MsgKey::RaScanForSyncthing) => "Scan in the Syncthing app to add this device",
        (Lang::ZhCn, MsgKey::RaScanForSyncthing) => "用 Syncthing 应用扫码添加这台设备",
    }
}
