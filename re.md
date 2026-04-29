### usb-map-gui

这是基于原始 `usb_map.cpp` 迁移出的 Rust + Slint 图形工具。它保留了原工具的两项核心能力，但把操作流程整理成了更直观的 GUI：

1. 扫描 `/dev` 下的串口设备，展示虚拟链接、物理设备和 USB 接口 ID 的对应关系。
2. 根据接口 ID 生成或更新 udev 规则，为设备创建稳定的 `/dev/<名称>` 符号链接。

### 使用方式

启动应用：

```bash
cargo run --release
```

界面分为上下两部分：

1. `设备列表`
   点击“刷新设备列表”后，会扫描 `ttyUSB*` 和 `ttyACM*` 设备，并显示：
   - 设备类型
   - 虚拟设备名
   - 物理设备名
   - 接口 ID
2. `规则表单`
   - 可以手动填写虚拟设备名称、物理接口 ID、规则文件路径
   - 也可以直接点击上方设备列表中的一行，把接口 ID 自动带入表单

默认规则文件路径：

```txt
/etc/udev/rules.d/relia.rules
```

生成的规则格式：

```txt
SUBSYSTEM=="tty", KERNELS=="<物理ID>", MODE:="0664", SYMLINK+="<虚拟名称>"
```

这里使用 `SUBSYSTEM=="tty"`，而不是原 C++ 示例里固定的 `KERNEL=="ttyUSB*"`，这样同一套规则即可覆盖 `ttyUSB*` 和 `ttyACM*` 设备。

### 保存规则后

将规则写入 `/etc/udev/rules.d/` 后，执行：

```bash
sudo udevadm control --reload-rules
sudo udevadm trigger
```

或者直接重启系统。

### 代码结构

```txt
ui/
  app_window.slint      主窗口布局
  device_table.slint    设备表格组件

src/
  main.rs               程序入口
  app_controller.rs     界面事件与状态流转
  ui_bindings.rs        Slint 属性读写与界面数据转换
  device_scan.rs        /dev 扫描与 libudev 查询
  rule_file.rs          udev 规则生成与更新
  models.rs             共享数据结构
```

### CI / Release

GitHub Actions 会在以下环境编译：

1. `ubuntu-22.04`
2. `ubuntu-24.04`

工作流特性：

1. 使用 `actions/cache` 缓存 Cargo 依赖和 `target`
2. 运行 `cargo build --release` 与 `cargo test`
3. 在推送 `v*` 标签时，将两个 Ubuntu 版本的编译产物上传到 GitHub Releases

### 链接方式说明

Rust crate 依赖会随可执行文件一起静态链接进产物；Linux 下保留为动态链接的部分仅为系统库，例如 `libc`、`libudev` 以及窗口系统相关库。
