# Distributed under the OSI-approved BSD 3-Clause License.  See accompanying
# file Copyright.txt or https://cmake.org/licensing for details.

cmake_minimum_required(VERSION 3.5)

file(MAKE_DIRECTORY
  "C:/Users/proxima/esp/v5.3/esp-idf/components/bootloader/subproject"
  "C:/programming/projects/rust/rt_audio_efect/esp_head_light/build/bootloader"
  "C:/programming/projects/rust/rt_audio_efect/esp_head_light/build/bootloader-prefix"
  "C:/programming/projects/rust/rt_audio_efect/esp_head_light/build/bootloader-prefix/tmp"
  "C:/programming/projects/rust/rt_audio_efect/esp_head_light/build/bootloader-prefix/src/bootloader-stamp"
  "C:/programming/projects/rust/rt_audio_efect/esp_head_light/build/bootloader-prefix/src"
  "C:/programming/projects/rust/rt_audio_efect/esp_head_light/build/bootloader-prefix/src/bootloader-stamp"
)

set(configSubDirs )
foreach(subDir IN LISTS configSubDirs)
    file(MAKE_DIRECTORY "C:/programming/projects/rust/rt_audio_efect/esp_head_light/build/bootloader-prefix/src/bootloader-stamp/${subDir}")
endforeach()
if(cfgdir)
  file(MAKE_DIRECTORY "C:/programming/projects/rust/rt_audio_efect/esp_head_light/build/bootloader-prefix/src/bootloader-stamp${cfgdir}") # cfgdir has leading slash
endif()
