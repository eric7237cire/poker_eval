# Using https://www.kaggle.com/code/androbomb/using-cnn-to-classify-images-w-pytorch
# and https://www.kaggle.com/code/gurpreetmeelu/pytorch-cnn-transfer-learning-image-classifier
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
import os
import random
import matplotlib.pyplot as plt
import seaborn as sns
import matplotlib.image as mpimg
from PIL import Image

from sklearn.metrics import confusion_matrix
from sklearn.metrics import classification_report

import torch
import torch.nn as nn
from torchvision import transforms
from torchvision import datasets
from torch.utils.data import DataLoader
from torchsummary import summary
from torchvision import models

from classify import prepare_data, read_classes, count_instances_per_class

cfg = EnvCfg()

def get_data():
    train_dir = cfg.CLASSIFY_DATA_PATH / cfg.TRAIN_FOLDER_NAME
    valid_dir = cfg.CLASSIFY_DATA_PATH / cfg.VALID_FOLDER_NAME

    image_transforms = {
        'train': transforms.Compose([
            transforms.Resize((224,224)),
            transforms.RandomHorizontalFlip(),
            transforms.ColorJitter(),
            transforms.RandomRotation(30),
            transforms.ToTensor(),
            transforms.Normalize([0.5, 0.5, 0.5], [0.5, 0.5, 0.5])
            
        ]),
        
        'valid': transforms.Compose([
            transforms.Resize((224,224)),
            transforms.RandomHorizontalFlip(),
            transforms.ColorJitter(),
            transforms.RandomRotation(30),
            transforms.ToTensor(),
            transforms.Normalize([0.5, 0.5, 0.5], [0.5, 0.5, 0.5])
        ])
    }

    data = {
        'train': datasets.ImageFolder(root=str(train_dir), transform=image_transforms['train']),
        'valid': datasets.ImageFolder(root=str(valid_dir), transform=image_transforms['valid'])
    }

    print("Training data: ", len(data['train']))
    print("Validation data: ", len(data['valid']))

    BATCH_SIZE = 32
    classes = read_classes()
    NUM_CLASSES = len(classes)

    train_loader = DataLoader(data['train'], batch_size = BATCH_SIZE, shuffle=True)
    valid_loader = DataLoader(data['valid'], batch_size = BATCH_SIZE, shuffle=True)

    return train_loader, valid_loader, NUM_CLASSES


def build_model(NUM_CLASSES:int):
    if not torch.cuda.is_available():
        raise Exception("No GPU found")
    
    device = torch.device("cuda:0")
    
    # Loading Pretrained Weights of Resnet50
    model = models.resnet50(weights='ResNet50_Weights.DEFAULT').to(device)
    summary(model, (3, 224, 224))

    # Freeze all layers except the last one
    for param in model.parameters():
        param.requires_grad = False

    num_features = model.fc.in_features
    model.fc = nn.Sequential(
        nn.Linear(num_features, 256),
        nn.ReLU(),
        nn.Dropout(p=0.5),
        nn.Linear(256, 128),
        nn.ReLU(),
        nn.Dropout(p=0.5),
        nn.Linear(128, 64),
        nn.ReLU(),
        nn.Dropout(p=0.5),
        nn.Linear(64, NUM_CLASSES)
    )

    # Transfer the model to device
    model = model.to(device)

    return model, device

def train_model(model, device, train_loader, valid_loader, NUM_CLASSES:int):
    criterion = torch.nn.CrossEntropyLoss()
    optimizer = torch.optim.SGD(model.fc.parameters(), lr=0.001, momentum=0.9)
    
    num_epochs = 20
    train_losses = []
    valid_losses = []

    for epoch in range(num_epochs):
        train_loss = 0
        valid_loss = 0

        # Training loop
        model.train()
        for images, labels in train_loader:
            images = images.to(device)
            labels = labels.to(device)
            optimizer.zero_grad()
            outputs = model(images)
            loss = criterion(outputs, labels)
            loss.backward()
            optimizer.step()
            train_loss += loss.item() * images.size(0)
            train_loss /= len(train_loader.dataset)
            train_losses.append(train_loss)
        
            # Validation loop
        model.eval()
        for images, labels in valid_loader:
            images = images.to(device)
            labels = labels.to(device)
            outputs = model(images)
            loss = criterion(outputs, labels)
            valid_loss += loss.item() * images.size(0)
            valid_loss /= len(valid_loader.dataset)
            valid_losses.append(valid_loss)
        
        # Print training and validation loss
        print(f'Epoch [{epoch + 1}/{num_epochs}], Train Loss: {train_loss:.4f}, Valid Loss: {valid_loss:.4f}')

    torch.save(model, cfg.PYTORCH_CNN_MODEL_PATH)

def test_model():
    if not torch.cuda.is_available():
        raise Exception("No GPU found")
    
    device = torch.device("cuda:0")
    
    test_transforms = transforms.Compose([
        transforms.Resize((224,224)),
        transforms.ToTensor(),
        transforms.Normalize([0.5, 0.5, 0.5], [0.5, 0.5, 0.5])
    ])

    # we don't have enough test data so use validation directory
    test_dir = cfg.CLASSIFY_DATA_PATH / cfg.VALID_FOLDER_NAME
    test_data = datasets.ImageFolder(str(test_dir), transform=test_transforms)
    test_loader = DataLoader(test_data, batch_size=32, shuffle=True)

    model = torch.load(cfg.PYTORCH_CNN_MODEL_PATH)
    model.eval()
    with torch.no_grad():
        correct = 0
        total = 0
        y_pred = []
        y_true = []
        for images, labels in test_loader:
            images = images.to(device)
            labels = labels.to(device)
            outputs = model(images)
            _, predicted = torch.max(outputs.data, 1)
            total += labels.size(0)
            correct += (predicted == labels).sum().item()
            y_pred.extend(predicted.cpu().numpy())
            y_true.extend(labels.cpu().numpy())
        print('Test Accuracy of the model on the test images: {} %'.format(100 * correct / total))

    # Plot confusion matrix
    cm = confusion_matrix(y_true, y_pred)
    # plt.figure(figsize=(10,10))
    # sns.heatmap(cm, annot=True, fmt='d', cmap='Blues', xticklabels=CLASS_NAMES, yticklabels=CLASS_NAMES)
    # plt.title('Confusion Matrix')
    # plt.ylabel('True Label')
    # plt.xlabel('Predicted Label')
    # plt.show()

    classes = read_classes()
    print(classification_report(y_true, y_pred, target_names=classes))

if __name__ == "__main__":
    
    # prepare_data(0.35, 0)

    count_instances_per_class()

    if False:
        train_loader, valid_loader, NUM_CLASSES = get_data()

        model, device = build_model(NUM_CLASSES)
        train_model(model, device, train_loader, valid_loader, NUM_CLASSES)

    test_model()
    # train_cnn()