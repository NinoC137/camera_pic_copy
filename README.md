# 相机SD照片自动化转存
## 背景
每次拍完照之后我都记不得第一张照片是从哪一个编号开始的了。。只能看照片修改日期，
然后手动选中后，从SD卡copy到电脑文件夹内。

这让我有两个很不爽的点：
1. 我要找到我这次拍到的那些照片
2. 手动copy费事，而且拷贝速度慢

## 功能描述：
配置好setting文件中的源文件夹、目标文件夹路径后，运行可执行程序便可以自动转存源文件夹中的所有符合格式要求的文件
，如尼康的NEF格式文件。并且，会自动记录最新转存的文件ID，以此实现自动同步最新的照片文件

在另一分支（MultiThread)中，实现了多线程支持，实测是比main分支的运行速度快很多的。

## 使用方法
1. 配置好setting.txt文件内容
> 如下所示 \
{\
"src_dir": "/Volumes/Nikon_32SD/DCIM/101NZ_30/", \
"dst_dir": "/Users/nino/Documents/Nikon/pictures", \
"log_path": "/Users/nino/Documents/Nikon/log.txt"\
}

2. 将setting.txt文件与可执行文件放置在同一文件夹内
3. 运行可执行文件，终端会实时打印log，可以通过ctrl+c关闭程序，程序一样会记录当前转存进度

## 解释说明
1. src_dir是源文件夹路径（只支持绝对路径），这里是我的SD卡插到读卡器连电脑后的路径。
2. dst_dir是目标文件夹（只支持绝对路径），要注意的是，
“/Users/nino/Documents/Nikon/”这一级路径文件夹必须要手动创建好（当然文件夹命名什么的可以不一样）
3. log_path也可以改，但建议同样放在“Nikon”这一级路径下

## 运行效果
终端会打印类似如下的log
> last id: 169 \
Copying "/Volumes/Nikon_32SD/DCIM/101NZ_30/DSC_0178.NEF" to "/Users/nino/Documents/Nikon/pictures/DSC_0178.NEF" \
Copying "/Volumes/Nikon_32SD/DCIM/101NZ_30/DSC_0180.NEF" to "/Users/nino/Documents/Nikon/pictures/DSC_0180.NEF" \
^C \
获取到中止指令，正在存储已转存的ID \
Saving last copied ID :180 \
update id : 180