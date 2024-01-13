# Using https://www.kaggle.com/code/androbomb/using-cnn-to-classify-images-w-pytorch
from pathlib import Path
import shutil
from PIL import Image
# Import PyTorch libraries
import torch
import torchvision
import torchvision.transforms as transforms
import torch.nn as nn
import torch.optim as optim
import torch.nn.functional as F
from env_cfg import EnvCfg

cfg = EnvCfg()


def prepare_data():
    
    print(f"Converting yolo dataset in [{cfg.CARD_YOLO_PATH}] to classification format in [{cfg.CNN_DATA_PATH}]")

    if cfg.CNN_DATA_PATH.exists():
        print(f"Removing {cfg.CNN_DATA_PATH}")
        shutil.rmtree(cfg.CNN_DATA_PATH)

    images_dir = cfg.CARD_YOLO_PATH / "images"
    labels_dir = cfg.CARD_YOLO_PATH / "labels"

    image_num = 0

    classes = read_classes()

    for label_file in labels_dir.iterdir():
        if label_file.suffix != ".txt":
            continue

        image_file = images_dir / label_file.with_suffix(".png").name

        img = Image.open(image_file)

        for line in label_file.open("r").readlines():
            fields = line.split(" ")
            card_class = classes[int(fields[0])]
            xs = []
            ys = []
            for i in range(0, 4):
                xs.append(float(fields[1 + i * 2]))
                ys.append(float(fields[2 + i * 2]))

            x_min = min(xs) 
            x_max = max(xs)
            y_min = min(ys)
            y_max = max(ys)

            width = x_max - x_min
            height = y_max - y_min

            x_min = x_min * img.width
            x_max = x_max * img.width
            y_min = y_min * img.height
            y_max = y_max * img.height

            # open the png image and crop this box out
            # Crop the image to the specified rectangle


            cropped_img = img.crop((x_min, y_min, x_max, y_max))

            # Resize the image to 128x128

            cropped_img = cropped_img.resize((128, 128))

            # Save the cropped image
            target_path = cfg.CNN_DATA_PATH / card_class / f"{image_num}.png"
            image_num += 1
            target_path.parent.mkdir(parents=True, exist_ok=True)
            cropped_img.save(target_path)

            print(f"Saved {target_path}, dimensions {cropped_img.width}x{cropped_img.height}")
            
            cropped_img.close()
        # Close the original and cropped images
        img.close()

    count_instances_per_class()

def read_classes():
    with open(cfg.CARD_YOLO_PATH / "classes.txt") as f:
        classes = f.read().strip().split("\n")

    if len(classes) != 52:
        raise Exception(f"Expected 52 classes, was {len(classes)}")
    
    return classes
    
def count_instances_per_class():
    
    classes = read_classes()
    count_class = {
        classes[k]: 0 for k in range(0, len(classes))
    }
    # now count how many of each class we have
    for class_dir in cfg.CNN_DATA_PATH.iterdir():
        if not class_dir.is_dir():
            continue

        num_files = len(list(class_dir.iterdir()))
        print(f"{class_dir.name}: num_files: {num_files}")
        count_class[class_dir.name] = num_files

    lowest_count = min( [count_class[c] for c in count_class] )
    lowest_cards = [c for c in count_class if count_class[c] == lowest_count]
    lowest_cards.sort()
    print(f"Lowest count: {lowest_count} for {lowest_cards}")


def load_dataset(data_path):
    
    # Load all the images
    transformation = transforms.Compose([
        # Randomly augment the image data
            # Random horizontal flip
        transforms.RandomHorizontalFlip(0.5),
            # Random vertical flip
        transforms.RandomVerticalFlip(0.3),
        # transform to tensors
        transforms.ToTensor(),
        # Normalize the pixel values (in R, G, and B channels)
        transforms.Normalize(mean=[0.5, 0.5, 0.5], std=[0.5, 0.5, 0.5])
    ])

    # Load all of the images, transforming them
    full_dataset = torchvision.datasets.ImageFolder(
        root=data_path,
        transform=transformation
    )
    
    
    # Split into training (70% and testing (30%) datasets)
    train_size = int(0.7 * len(full_dataset))
    test_size = len(full_dataset) - train_size
    
    # use torch.utils.data.random_split for training/test split
    train_dataset, test_dataset = torch.utils.data.random_split(full_dataset, [train_size, test_size])
    
    # define a loader for the training data we can iterate through in 50-image batches
    train_loader = torch.utils.data.DataLoader(
        train_dataset,
        batch_size=50,
        num_workers=0,
        shuffle=False
    )
    
    # define a loader for the testing data we can iterate through in 50-image batches
    test_loader = torch.utils.data.DataLoader(
        test_dataset,
        batch_size=50,
        num_workers=0,
        shuffle=False
    )
        
    return train_loader, test_loader


# Create a neural net class
class Net(nn.Module):
    
    
    # Defining the Constructor
    def __init__(self, num_classes=3):
        super(Net, self).__init__()
        
        # In the init function, we define each layer we will use in our model
        
        # Our images are RGB, so we have input channels = 3. 
        # We will apply 12 filters in the first convolutional layer
        self.conv1 = nn.Conv2d(in_channels=3, out_channels=12, kernel_size=3, stride=1, padding=1)
        
        # A second convolutional layer takes 12 input channels, and generates 24 outputs
        self.conv2 = nn.Conv2d(in_channels=12, out_channels=24, kernel_size=3, stride=1, padding=1)
        
        # We in the end apply max pooling with a kernel size of 2
        self.pool = nn.MaxPool2d(kernel_size=2)
        
        # A drop layer deletes 20% of the features to help prevent overfitting
        self.drop = nn.Dropout2d(p=0.2)
        
        # Our 128x128 image tensors will be pooled twice with a kernel size of 2. 128/2/2 is 32.
        # This means that our feature tensors are now 32 x 32, and we've generated 24 of them
        
        # We need to flatten these in order to feed them to a fully-connected layer
        self.fc = nn.Linear(in_features=32 * 32 * 24, out_features=num_classes)

    def forward(self, x):
        # In the forward function, pass the data through the layers we defined in the init function
        
        # Use a ReLU activation function after layer 1 (convolution 1 and pool)
        x = F.relu(self.pool(self.conv1(x))) 
        
        # Use a ReLU activation function after layer 2
        x = F.relu(self.pool(self.conv2(x)))  
        
        # Select some features to drop to prevent overfitting (only drop during training)
        x = F.dropout(self.drop(x), training=self.training)
        
        # Flatten
        x = x.view(-1, 32 * 32 * 24)
        # Feed to fully-connected layer to predict class
        x = self.fc(x)
        # Return class probabilities via a log_softmax function 
        return torch.log_softmax(x, dim=1)
    
def train(model, device, train_loader, optimizer, epoch, loss_criteria):
    # Set the model to training mode
    model.train()
    train_loss = 0
    print("Epoch:", epoch)
    batch_idx =  0
    # Process the images in batches
    for batch_idx, (data, target) in enumerate(train_loader):
        # Use the CPU or GPU as appropriate
        # Recall that GPU is optimized for the operations we are dealing with
        data, target = data.to(device), target.to(device)
        
        # Reset the optimizer
        optimizer.zero_grad()
        
        # Push the data forward through the model layers
        output = model(data)
        
        # Get the loss
        loss = loss_criteria(output, target)

        # Keep a running total
        train_loss += loss.item()
        
        # Backpropagate
        loss.backward()
        optimizer.step()
        
        # Print metrics so we see some progress
        print('\tTraining batch {} Loss: {:.6f}'.format(batch_idx + 1, loss.item()))
            
    # return average loss for the epoch
    avg_loss = train_loss / (batch_idx+1)
    print('Training set: Average loss: {:.6f}'.format(avg_loss))
    return avg_loss    


def test(model, device, test_loader, loss_criteria):
    # Switch the model to evaluation mode (so we don't backpropagate or drop)
    model.eval()
    test_loss = 0
    correct = 0
    with torch.no_grad():
        batch_count = 0
        for data, target in test_loader:
            batch_count += 1
            data, target = data.to(device), target.to(device)
            
            # Get the predicted classes for this batch
            output = model(data)
            
            # Calculate the loss for this batch
            test_loss += loss_criteria(output, target).item()
            
            # Calculate the accuracy for this batch
            _, predicted = torch.max(output.data, 1)
            correct += torch.sum(target==predicted).item()

    # Calculate the average loss and total accuracy for this epoch
    avg_loss = test_loss / batch_count
    print('Validation set: Average loss: {:.6f}, Accuracy: {}/{} ({:.0f}%)\n'.format(
        avg_loss, correct, len(test_loader.dataset),
        100. * correct / len(test_loader.dataset)))
    
    # return average loss for the epoch
    return avg_loss


def train_cnn():
    train_loader, test_loader = load_dataset(cfg.CNN_DATA_PATH)
    batch_size = train_loader.batch_size
    print("Data loaders ready to read", cfg.CNN_DATA_PATH)
    print("Train:", len(train_loader.dataset))
    print("Batch size:", batch_size)

    
    device = "cpu"
    if (torch.cuda.is_available()):
        # if GPU available, use cuda (on a cpu, training will take a considerable length of time!)
        device = "cuda"
    else:
        raise Exception("No GPU found")
    
    classes = read_classes()

    # Create an instance of the model class and allocate it to the device
    model = Net(num_classes=len(classes)).to(device)

    print(model)

    # Use an "Adam" optimizer to adjust weights
    optimizer = optim.Adam(model.parameters(), lr=0.01)

    # Specify the loss criteria
    loss_criteria = nn.CrossEntropyLoss()

    # Track metrics in these arrays
    epoch_nums = []
    training_loss = []
    validation_loss = []

    # Train over 10 epochs (We restrict to 10 for time issues)
    epochs = 200
    print('Training on', device)
    for epoch in range(1, epochs + 1):
            train_loss = train(model, device, train_loader, optimizer, epoch, loss_criteria)
            test_loss = test(model, device, test_loader, loss_criteria)
            epoch_nums.append(epoch)
            training_loss.append(train_loss)
            validation_loss.append(test_loss)

if __name__ == "__main__":
    # prepare_data()

    count_instances_per_class()

    # train_cnn()