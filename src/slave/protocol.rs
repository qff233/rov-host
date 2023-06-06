// 主界面
pub const METHOD_GET_INFO: &'static str                           = "get_info";                           // 获取信息（舱内温度、航向角等）
pub const METHOD_MOVE: &'static str                               = "move";                               // 移动
pub const METHOD_SET_DEPTH_LOCKED: &'static str                   = "set_depth_locked";                   // 开启/关闭深度锁定
pub const METHOD_SET_DIRECTION_LOCKED: &'static str               = "set_direction_locked";               // 开启/关闭方向锁定
pub const METHOD_CATCH: &'static str                              = "catch";                              // 控制机械臂张合
// 调试界面
pub const METHOD_SET_DEBUG_MODE_ENABLED: &'static str             = "set_debug_mode_enabled";             // 开启/关闭调试模式
pub const METHOD_GET_FEEDBACKS: &'static str                      = "get_feedbacks";                      // 请求反馈信息
pub const METHOD_SET_PROPELLER_PWM_FREQ_CALIBRATION: &'static str = "set_propeller_pwm_freq_calibration"; // 推进器 PWM 频率校准
pub const METHOD_SET_PROPELLER_PARAMETERS: &'static str           = "set_propeller_parameters";           // 推进器参数
pub const METHOD_SET_CONTROL_LOOP_PARAMETERS: &'static str        = "set_control_loop_parameters";        // 控制环参数
pub const METHOD_SAVE_PARAMETERS: &'static str                    = "save_parameters";                    // 保存参数
pub const METHOD_LOAD_PARAMETERS: &'static str                    = "load_parameters";                    // 读取参数
pub const METHOD_SET_PROPELLER_VALUES: &'static str               = "set_propeller_values";               // 设置推进器输出
// 固件更新界面
pub const METHOD_UPDATE_FIRMWARE: &'static str                    = "update_firmware";                    // 固件更新