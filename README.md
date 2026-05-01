# usb-map-gui

基于上游项目 [Mq-b/usb_map](https://github.com/Mq-b/usb_map) 改写的 Rust + Slint 图形界面版本。

上游仓库是一个 Linux 下基于 `udev` 的 USB 串口工具集，包含：

1. `usb_map`
   用于查询 USB 串口设备映射关系，并生成持久化的 udev 规则。
2. `find_4g_module`
   用于扫描串口并定位 4G 模块。

当前这个 Rust 项目只迁移了其中的 `usb_map` 功能，没有实现 `find_4g_module`。

![usb-map-gui](./images/usb-map-gui.png)

## 功能

本项目提供一个单一用途的 GUI 工具，用于：

1. 扫描 `/dev` 下的 `ttyUSB*`、`ttyACM*` 以及相关符号链接
2. 读取设备对应的 USB 接口 ID
3. 在界面中展示“虚拟设备 / 物理设备 / 接口 ID”映射关系
4. 生成或更新 udev 规则文件，为设备创建稳定的 `/dev/<name>` 符号链接

## 和上游项目的关系

本仓库的实现依据主要来自上游仓库中的：

1. `src/usb_map.cpp` 的设备扫描与规则生成逻辑
2. 上游 `README.md` 中对 `usb_map` 命令行行为的说明

与上游相比，本项目的主要变化是：

1. 从 C++ 终端程序改为 Rust + Slint GUI
2. 只保留 `usb_map` 这一项能力
3. 将界面、事件绑定、业务逻辑拆分为多个模块，便于维护
4. 在规则生成时使用 `SUBSYSTEM=="tty"`，同时兼容 `ttyUSB*` 与 `ttyACM*`

## 运行环境

适用平台：

1. Linux
2. 存在 `/dev`、`udev`、`libudev`

如果要把规则写入 `/etc/udev/rules.d/`，通常需要足够的文件写入权限。

## 启动

```bash
cargo run --release
```

启动后界面分为两部分：

1. `设备列表`
   用来刷新和查看当前串口映射关系
2. `规则表单`
   用来生成或更新 udev 规则

点击设备列表中的某一行后，界面会自动把该行的接口 ID 带入下方表单。

## 规则格式

程序写入的规则格式为：

```txt
SUBSYSTEM=="tty", KERNELS=="<物理ID>", MODE:="0664", SYMLINK+="<虚拟名称>"
```

保存规则后可执行：

```bash
sudo udevadm control --reload-rules
sudo udevadm trigger
```

或重启系统使规则生效。

## 项目结构

```txt
ui/
  app_window.slint      主窗口布局
  device_table.slint    设备列表组件

src/
  main.rs               程序入口
  app_controller.rs     界面事件和状态流转
  ui_bindings.rs        Slint 属性绑定与界面数据转换
  device_scan.rs        /dev 扫描与 libudev 查询
  rule_file.rs          udev 规则生成与更新
  models.rs             共享数据结构
```

## CI / Release

仓库内置 GitHub Actions 工作流，负责：

1. 在 `ubuntu-22.04` 和 `ubuntu-24.04` 上编译
2. 使用 `actions/cache` 缓存 Cargo 依赖与 `target`
3. 运行 `cargo test`
4. 在推送 `v*` 标签时，将构建产物上传到 GitHub Releases

## 链接说明

Rust crate 依赖会随最终可执行文件一起静态链接到产物中；Linux 下保留为动态链接的部分仅为系统库，例如 `libc`、`libudev` 以及窗口系统相关库。

## 上游项目

上游仓库地址：

`https://github.com/Mq-b/usb_map`
