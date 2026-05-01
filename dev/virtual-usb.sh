#!/usr/bin/env bash
set -euo pipefail

DEVICES=(
    "ttyUSB0 188 0"
    "ttyUSB1 188 1"
    "ttyACM0 166 0"
)

SYMLINKS=(
    "ttyMyDevice ttyUSB0"
    "ttyModem ttyACM0"
)

usage() {
    echo "用法: $0 {create|remove|status}"
    echo ""
    echo "  create  创建虚拟 USB 串口设备（需要 sudo）"
    echo "  remove  删除虚拟设备（需要 sudo）"
    echo "  status  查看当前虚拟设备状态"
}

create_devices() {
    echo "创建虚拟物理设备..."
    for entry in "${DEVICES[@]}"; do
        read -r name major minor <<< "$entry"
        path="/dev/$name"
        if [ -e "$path" ]; then
            echo "  跳过 $path（已存在）"
        else
            sudo mknod "$path" c "$major" "$minor"
            echo "  创建 $path"
        fi
    done

    echo "创建虚拟符号链接..."
    for entry in "${SYMLINKS[@]}"; do
        read -r link target <<< "$entry"
        path="/dev/$link"
        if [ -L "$path" ]; then
            echo "  跳过 $path（已存在）"
        else
            sudo ln -s "$target" "$path"
            echo "  创建 $path -> $target"
        fi
    done

    echo "完成。"
}

remove_devices() {
    echo "删除虚拟符号链接..."
    for entry in "${SYMLINKS[@]}"; do
        read -r link target <<< "$entry"
        path="/dev/$link"
        if [ -L "$path" ]; then
            sudo rm "$path"
            echo "  删除 $path"
        else
            echo "  跳过 $path（不存在）"
        fi
    done

    echo "删除虚拟物理设备..."
    for entry in "${DEVICES[@]}"; do
        read -r name major minor <<< "$entry"
        path="/dev/$name"
        if [ -e "$path" ]; then
            sudo rm "$path"
            echo "  删除 $path"
        else
            echo "  跳过 $path（不存在）"
        fi
    done

    echo "完成。"
}

show_status() {
    echo "虚拟设备状态："
    echo ""
    echo "物理设备："
    for entry in "${DEVICES[@]}"; do
        read -r name major minor <<< "$entry"
        path="/dev/$name"
        if [ -e "$path" ]; then
            echo "  ✓ $path"
        else
            echo "  ✗ $path（不存在）"
        fi
    done

    echo ""
    echo "符号链接："
    for entry in "${SYMLINKS[@]}"; do
        read -r link target <<< "$entry"
        path="/dev/$link"
        if [ -L "$path" ]; then
            echo "  ✓ $path -> $(readlink "$path")"
        else
            echo "  ✗ $path（不存在）"
        fi
    done
}

if [ $# -lt 1 ]; then
    usage
    exit 1
fi

case "$1" in
    create) create_devices ;;
    remove) remove_devices ;;
    status) show_status ;;
    *) usage; exit 1 ;;
esac
