use bitflags::bitflags;
use serde::Deserialize;
use std::io::Read;

bitflags! {
    #[derive(Deserialize, Debug, Copy, Clone)]
    struct ArchiveFlags: u32 {
        const HasDirNames = 0x01;
        const HasFileNames = 0x02;
        const CompressedByDefault = 0x4;
        const Unknown = 0x08;
        const Unknown2 = 0x10;
        const Unknown3 = 0x20;
        const BigEndien = 0x40;
        const Unkown4 = 0x100;
        const Unkown5 = 0x200;
        const Unkown6 = 0x400;
    }
}

bitflags! {
    #[derive(Deserialize, Debug, Copy, Clone)]
    struct ContentFlags: u32 {
        const MESHES = 0x01;
        const TEXTURES = 0x02;
        const MENUS = 0x04;
        const SOUNDS = 0x08;
        const VOICES = 0x10;
        const SHADERS = 0x20;
        const TREES = 0x40;
        const FONTS = 0x80;
        const MISC = 0x100;
    }
}

#[repr(C, packed)]
#[derive(Deserialize, Debug, Copy, Clone)]
struct BSAHeader {
    magic_num: [char; 4],
    version: u32,
    offset_to_folders: u32,
    archive_flags: ArchiveFlags,
    folder_count: u32,
    total_file_count: u32,
    total_folder_name_length: u32,
    total_file_name_length: u32,
    content_flags: ContentFlags,
}

// #[repr(C, packed)]
#[derive(Deserialize, Debug)]
struct FolderRecord {
    folder_hash: u64,
    folder_file_count: u32,
    start_offset: u32,
}

// #[repr(C, packed)]
#[derive(Deserialize, Debug)]
struct FileRecord {
    file_name_hash: u64,
    file_size: u32,
    offset: u32,
}

#[derive(Default)]
struct FolderContents {
    name: String,
    file_records: Vec<FileRecord>,
}

struct FileNames {
    names: Vec<String>,
}

struct CompressedFileBlock {
    path: String,
    uncompressed_size: u32,
    data: Vec<u8>,
}

struct UncompressedFileBlock {
    path: String,
    data: Vec<u8>,
}

fn decode_folders(data: &[u8], header: BSAHeader) -> Vec<FolderRecord> {
    let mut records = Vec::new();
    for chunk in data.chunks(size_of::<FolderRecord>()) {
        records.push(bincode::deserialize(chunk).unwrap());
    }
    records
}

struct File {
    hash: u64,
    path: String,
    data: Vec<u8>,
}

#[derive(Default)]
struct Folder {
    hash: u64,
    files: Vec<File>,
}

fn parse_bz_string(data: &[u8]) -> (String, usize) {
    let mut string = String::new();
    let size = data[0] as usize;
    for i in 0..size {
        string.push(data[i + 1] as char);
    }

    (string, size + 1)
}

fn decode_file_records(
    data: &[u8],
    folder_records: &[FolderRecord],
    header: BSAHeader,
) -> Vec<FolderContents> {
    let mut folder_contents = Vec::new();
    let mut offset_to_file = header.offset_to_folders as usize
        + header.folder_count as usize * size_of::<FolderRecord>();
    for folder_record in folder_records {
        // let records = &data[folder_record.start_offset as usize..];
        let mut file_records: Vec<FileRecord> = Vec::new();
        // let mut name = String::new();
        let mut offset = folder_record.start_offset as usize;
        // TODO: offset_to_file correctly points us to the start of a file record section, the offset we get from the folder_record points to the name section
        let (name, record_offset) = parse_bz_string(&data[offset..]);
        for i in 0..folder_record.folder_file_count {
            let record_start = offset_to_file as usize;
            let record_end = record_start + size_of::<FileRecord>();
            let file_record: FileRecord =
                bincode::deserialize(&data[record_start..record_end]).unwrap();
            file_records.push(file_record);
            offset_to_file += size_of::<FolderRecord>();
        }
        folder_contents.push(FolderContents { name, file_records });
    }

    folder_contents
}

fn main() {
    let meshes_bin =
        std::fs::read("E:/Programs/Steam/steamapps/common/Oblivion/Data/Oblivion - Meshes.bsa")
            .unwrap();

    let header: BSAHeader = bincode::deserialize(&meshes_bin[..size_of::<BSAHeader>()]).unwrap();
    println!("FolderRecord size {}", size_of::<FolderRecord>());
    println!("FileRecord size {}", size_of::<FileRecord>());

    let folder_record_start = header.offset_to_folders as usize;
    let folder_record_end =
        folder_record_start + header.folder_count as usize * size_of::<FolderRecord>();
    let folder_record_region = &meshes_bin[folder_record_start..folder_record_end];

    let folder_records = decode_folders(folder_record_region, header);

    let file_record_start = folder_record_end;
    let file_record_end =
        file_record_start + header.total_file_count as usize * size_of::<FileRecord>();

    let file_record_region = &meshes_bin[file_record_start..file_record_end];

    let folder_contents =
        decode_file_records(meshes_bin.as_slice(), folder_records.as_slice(), header);
    println!("{:?}", folder_contents[0].name);
    // println!("{:?}", folder_records);
    // println!("{:?}", file_records);
    let mut total: usize = 0;
    // for r in file_records {
    //     // println!("{}", r.file_size & (!0x80000000));
    //     total += (r.file_size & (!0xC0000000)) as usize;
    // }
    println!("total {}", total);
}
