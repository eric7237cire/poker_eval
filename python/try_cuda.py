# from numba import cuda
# print(cuda.gpus)

# import tensorflow as tf

# # Check if TensorFlow is built with CUDA (GPU support)
# print("Is Built with CUDA:", tf.test.is_built_with_cuda())

# # Check if a GPU device is available and TensorFlow can access it
# print("Is GPU available:", tf.config.list_physical_devices('GPU'))

# # Print the version of TensorFlow
# print("TensorFlow Version:", tf.__version__)

import torch

# Check if CUDA is available and which version of cuDNN is installed
print("Is CUDA available:", torch.cuda.is_available())
print("cuDNN Version:", torch.backends.cudnn.version())

# List available CUDA devices (if any)
print("Available CUDA Devices:", torch.cuda.device_count())
for i in range(torch.cuda.device_count()):
    print(f"CUDA Device {i}:", torch.cuda.get_device_name(i))

